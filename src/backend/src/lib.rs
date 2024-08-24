use ::tracing::info;
use chats::{Chat, ChatRepository, CreateChatCommand};
use serde::Deserialize;
use tracing::warn;
use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime},
    prelude::*,
};
use tracing_web::{performance_layer, MakeConsoleWriter};
use worker::*;

mod chatroom;
mod chats;
mod messaging;

#[derive(Deserialize)]
struct QueryStringParameters {
    password: String,
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
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let database_binding = env.d1("CHAT_METADATA").map_err(|e| {
        warn!("{}", e);
        worker::Error::RustError("CHAT_METADATA binding not found".to_string())
    })?;

    Router::with_data(AppState {
        chat_repository: ChatRepository::new(database_binding),
    })
    .on_async("/api/connect/:chat_id", handle_message_websocket)
    .get_async("/api/message/:chat_id", handle_message)
    .put_async("/api/message/:chat_id", handle_message)
    .post_async("/api/message/:chat_id", handle_message)
    .delete_async("/api/message/:chat_id", handle_message)
    .get_async("/api/chats", handle_get_active_chats)
    .get_async("/api/chats/:chat_id", handle_get_specific_chat)
    .post_async("/api/chats", handle_create_new_chat)
    .run(req, env)
    .await
}

pub async fn handle_get_active_chats(
    _req: Request,
    ctx: RouteContext<AppState>,
) -> Result<Response> {
    let chats = ctx.data.chat_repository.list_all_chats(10).await;

    Response::from_json(&chats)
}

pub async fn handle_create_new_chat(
    mut req: Request,
    ctx: RouteContext<AppState>,
) -> Result<Response> {
    let command: CreateChatCommand = req.json().await.unwrap();

    let chat = Chat::new(command.name, command.password);

    let chat = ctx
        .data
        .chat_repository
        .add_chat(chat)
        .await
        .map_err(|_e| Error::RustError("Failure creating chat".to_string()))?;

    Response::from_json(&chat)
}

pub async fn handle_get_specific_chat(
    _req: Request,
    ctx: RouteContext<AppState>,
) -> Result<Response> {
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

pub async fn handle_message(req: Request, ctx: RouteContext<AppState>) -> Result<Response> {
    if let Some(chat_id) = ctx.param("chat_id") {
        let object = ctx.durable_object("CHATROOM").unwrap();
        let id = object.id_from_name(chat_id.as_str()).unwrap();
        let stub = id.get_stub().unwrap();
        let res = stub.fetch_with_request(req.clone().unwrap()).await.unwrap();

        return Ok(res);
    }

    Response::error("Bad Request", 400)
}

pub async fn handle_message_websocket(
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

        let password = password_header.unwrap();

        info!("Password is '{}'", &password.password);

        let is_password_valid = ctx
            .data
            .chat_repository
            .validate_password(chat_id, &password.password)
            .await;

        info!("Password match - {is_password_valid}");

        if !is_password_valid {
            return Ok(Response::builder()
                .with_status(401)
                .body(ResponseBody::Empty));
        }

        let object = ctx.durable_object("CHATROOM").unwrap();
        let id = object.id_from_name(chat_id.as_str()).unwrap();
        let stub = id.get_stub().unwrap();
        let res = stub.fetch_with_request(req.clone().unwrap()).await.unwrap();

        return Ok(res);
    }

    Response::error("Bad Request", 400)
}
