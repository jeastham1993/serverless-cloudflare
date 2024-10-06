use auth::{AuthenticationService, Claims};
use chats::{Chat, ChatRepository, CreateChatCommand};
use serde::Deserialize;
use tracing::{info, warn};
use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime},
    prelude::*,
};
use tracing_web::{performance_layer, MakeConsoleWriter};
use worker::*;
use worker::postgres_tls::PassthroughTls;

mod auth;
mod chatroom;
mod chats;
mod messaging;

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
    auth_service: AuthenticationService
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let database_binding = env.d1("CHAT_METADATA").map_err(|e| {
        warn!("{}", e);
        worker::Error::RustError("CHAT_METADATA binding not found".to_string())
    })?;

    let jwt_secret = env.secret("JWT_SECRET")?.to_string();

    Router::with_data(AppState {
        chat_repository: ChatRepository::new(database_binding),
        auth_service: AuthenticationService::new(jwt_secret)
        
    })
    .on_async("/api/connect/:chat_id", handle_websocket_connect)
    .get_async("/api/chats", handle_get_active_chats)
    .get_async("/api/chats/:chat_id", handle_get_specific_chat)
    .post_async("/api/chats", handle_create_new_chat)
    .run(req, env)
    .await
}

pub async fn handle_get_active_chats(
    req: Request,
    ctx: RouteContext<AppState>,
) -> Result<Response> {
    if let Err(_) = verify_jwt(&req, &ctx.data.auth_service) {
        return Response::error("Unauthorized", 401);
    }

    let chats = ctx.data.chat_repository.list_all_chats(10).await;

    Response::from_json(&chats)
}

pub async fn handle_create_new_chat(
    mut req: Request,
    ctx: RouteContext<AppState>,
) -> Result<Response> {
    let claims = match verify_jwt(&req, &ctx.data.auth_service) {
        Ok(claims) => claims,
        Err(_) => return Response::error("Unauthorized", 401),
    };

    let command: CreateChatCommand = req.json().await.unwrap();

    let chat = Chat::new(command.name, claims.sub);

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
    if let Err(_) = verify_jwt(&req, &ctx.data.auth_service) {
        return Response::error("Unauthorized", 401);
    }

    if let Some(chat_id) = ctx.param("chat_id") {
        let chat = ctx
            .data
            .chat_repository
            .get_chat(chat_id)
            .await
            .map_err(|_e| Error::RustError("Failure retrieving chat".to_string()))?;

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

        match &ctx
            .data
            .auth_service
            .verify_jwt_token(&password_header.unwrap().key)
        {
            Ok(claims) => {
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
            Err(_) => return Response::error("Unauthorized", 401),
        };
    }

    Response::error("Bad Request", 400)
}

pub fn verify_jwt(req: &Request, auth_service: &AuthenticationService) -> Result<Claims> {
    tracing::info!("Verifying JWT");
    let auth_header = req
        .headers()
        .get("Authorization")?
        .ok_or(Error::RustError("No Authorization header".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(Error::RustError("Invalid Authorization header".into()))?;

    let token = auth_service
        .verify_jwt_token(token)
        .map_err(|_e| Error::RustError("Failure verifying token".to_string()))?;

    Ok(token)
}
