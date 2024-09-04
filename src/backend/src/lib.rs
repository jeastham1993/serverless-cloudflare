use chats::{Chat, ChatRepository, CreateChatCommand};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tracing::warn;
use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime},
    prelude::*,
};
use tracing_web::{performance_layer, MakeConsoleWriter};
use users::{User, UserRepository};
use worker::*;

mod chatroom;
mod chats;
mod messaging;
mod users;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Deserialize)]
struct QueryStringParameters {
    key: String,
}

#[event(start)]
fn start() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false) // Only partially supported across JavaScript runtimes
        .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
        .with_writer(MakeConsoleWriter); // write events to the console
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();
}

pub struct AppState {
    chat_repository: ChatRepository,
    user_repository: UserRepository,
    jwt_secret: String,
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let database_binding = env.d1("CHAT_METADATA").map_err(|e| {
        warn!("{}", e);
        worker::Error::RustError("CHAT_METADATA binding not found".to_string())
    })?;

    let users_database_binding = env.d1("CHAT_METADATA").map_err(|e| {
        warn!("{}", e);
        worker::Error::RustError("CHAT_METADATA binding not found".to_string())
    })?;

    let jwt_secret = env.secret("JWT_SECRET")?.to_string();

    Router::with_data(AppState {
        chat_repository: ChatRepository::new(database_binding),
        user_repository: UserRepository::new(users_database_binding),
        jwt_secret,
    })
    .post_async("/api/register", handle_register)
    .post_async("/api/login", handle_login)
    .on_async("/api/connect/:chat_id", handle_websocket_connect)
    .get_async("/api/chats", handle_get_active_chats)
    .get_async("/api/chats/:chat_id", handle_get_specific_chat)
    .post_async("/api/chats", handle_create_new_chat)
    .run(req, env)
    .await
}

pub async fn handle_register(mut req: Request, ctx: RouteContext<AppState>) -> Result<Response> {
    #[derive(Deserialize)]
    struct RegisterCommand {
        username: String,
        password: String,
    }

    let command: RegisterCommand = req.json().await?;
    let user = User::new(command.username, command.password);

    match ctx.data.user_repository.add_user(user).await {
        Ok(_) => Response::ok("User registered successfully"),
        Err(_) => Response::error("Failed to register user", 500),
    }
}

pub async fn handle_login(mut req: Request, ctx: RouteContext<AppState>) -> Result<Response> {
    #[derive(Deserialize)]
    struct LoginCommand {
        username: String,
        password: String,
    }

    let command: LoginCommand = req.json().await?;
    
    if let Ok(user) = ctx.data.user_repository.get_user(&command.username).await {
        if user.verify_password(&command.password) {
            let claims = Claims {
                sub: user.username,
                exp: (Date::now().as_millis() / 1000) as usize + 3600, // 1 hour expiration
            };

            let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(ctx.data.jwt_secret.as_ref()))
                .map_err(|e| Error::RustError(e.to_string()))?;
            return Response::ok(token);
        }
    }

    Response::error("Invalid credentials", 401)
}

pub async fn handle_get_active_chats(
    req: Request,
    ctx: RouteContext<AppState>,
) -> Result<Response> {
    if let Err(_) = verify_jwt(&req, &ctx.data.jwt_secret) {
        return Response::error("Unauthorized", 401);
    }

    let chats = ctx.data.chat_repository.list_all_chats(10).await;

    Response::from_json(&chats)
}

pub async fn handle_create_new_chat(
    mut req: Request,
    ctx: RouteContext<AppState>,
) -> Result<Response> {
    let claims = match verify_jwt(&req, &ctx.data.jwt_secret) {
        Ok(claims) => claims,
        Err(_) => return Response::error("Unauthorized", 401),
    };

    let command: CreateChatCommand = req.json().await.unwrap();

    let chat = Chat::new(command.name, command.password, claims.sub);

    let chat = ctx
        .data
        .chat_repository
        .add_chat(chat)
        .await
        .map_err(|_e| Error::RustError("Failure creating chat".to_string()))?;

    Response::from_json(&chat)
}

pub async fn handle_get_specific_chat(
    req: Request,
    ctx: RouteContext<AppState>,
) -> Result<Response> {
    if let Err(_) = verify_jwt(&req, &ctx.data.jwt_secret) {
        return Response::error("Unauthorized", 401);
    }

    if let Some(chat_id) = ctx.param("chat_id") {
        let chat = ctx
            .data
            .chat_repository
            .get_chat(chat_id)
            .await
            .map_err(|_e| Error::RustError("Failure creating chat".to_string()))?;

        return Response::from_json(&chat);
    }

    Ok(Response::builder()
        .with_status(40)
        .body(ResponseBody::Empty))
}

pub async fn handle_websocket_connect(
    req: Request,
    ctx: RouteContext<AppState>,
) -> Result<Response> {
    let upgrade_header = req.headers().get("Upgrade");

    if upgrade_header.is_err() || upgrade_header.unwrap().unwrap() != "websocket" {
        return Ok(Response::builder()
            .with_status(426)
            .body(ResponseBody::Empty));
    }

    if let Some(chat_id) = ctx.param("chat_id") {
        let password_header = req.query::<QueryStringParameters>();

        if password_header.is_err() {
            return Ok(Response::builder()
                .with_status(401)
                .body(ResponseBody::Empty));
        }

        let claims = match verify_jwt_token(&password_header.unwrap().key, &ctx.data.jwt_secret) {
            Ok(claims) => claims,
            Err(_) => return Response::error("Unauthorized", 401),
        };

        let url = req.url()?;
        let mut new_url = url.clone();
        new_url.set_query(Some(&format!("user_id={}", claims.sub)));
        let mut new_req = Request::new(new_url.as_str(), req.method())?;
        let _ = new_req.headers_mut()?.set("Upgrade", "websocket");

        let object = ctx.durable_object("CHATROOM").unwrap();
        let id = object.id_from_name(chat_id.as_str()).unwrap();
        let stub = id.get_stub().unwrap();
        let res = stub.fetch_with_request(new_req).await.unwrap();

        return Ok(res);
    }

    Response::error("Bad Request", 400)
}

fn verify_jwt(req: &Request, secret: &str) -> Result<Claims> {
    tracing::info!("Verifying JWT");
    let auth_header = req.headers().get("Authorization")?.ok_or(Error::RustError("No Authorization header".into()))?;

    let token = auth_header.strip_prefix("Bearer ").ok_or(Error::RustError("Invalid Authorization header".into()))?;
    
    verify_jwt_token(token, secret)
}

fn verify_jwt_token(token: &str, secret: &str) -> Result<Claims> {
    tracing::info!("Verifying JWT");
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default())
        .map_err(|e| Error::RustError(e.to_string()))?;
    Ok(token_data.claims)
}