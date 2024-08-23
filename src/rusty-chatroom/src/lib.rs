use ::tracing::info;
use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime},
    prelude::*,
};
use tracing_web::{performance_layer, MakeConsoleWriter};
use worker::*;

mod chatroom;
mod html;
mod messaging;

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

pub struct AppState {}

#[event(fetch)]
async fn fetch(req: Request, env: Env, ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    info!("Ready...");

    Router::with_data(AppState {})
        .get("/", handle_home_get)
        .get("/style.css", handle_get_css)
        .get("/app.js", handle_get_js)
        .on_async("/connect/:chat_id", handle_message_websocket)
        .options("/message/:chat_id", handle_options)
        .get_async("/message/:chat_id", handle_message)
        .put_async("/message/:chat_id", handle_message)
        .post_async("/message/:chat_id", handle_message)
        .delete_async("/message/:chat_id", handle_message)
        .run(req, env)
        .await
}

pub fn handle_home_get(mut req: Request, ctx: RouteContext<AppState>) -> Result<Response> {
    let html_string = html::load_html();

    Response::from_html(html_string)
}

pub fn handle_get_css(mut req: Request, ctx: RouteContext<AppState>) -> Result<Response> {
    let css_string = html::load_css();

    Response::from_body(ResponseBody::Body(css_string.into_bytes()))
}

pub fn handle_get_js(mut req: Request, ctx: RouteContext<AppState>) -> Result<Response> {
    let js_string = html::load_js();

    Response::from_body(ResponseBody::Body(js_string.into_bytes()))
}

pub fn handle_options(mut req: Request, ctx: RouteContext<AppState>) -> Result<Response> {
    let mut headers = Headers::new();
    headers.append("Access-Control-Allow-Origin", "*");
    headers.append("Access-Control-Allow-Methods", "GET, HEAD, POST, OPTIONS");
    headers.append("Access-Control-Allow-Headers", "Content-Type");

    Ok(Response::builder()
        .with_headers(headers)
        .with_status(200)
        .body(ResponseBody::Empty))
}

pub async fn handle_message(mut req: Request, ctx: RouteContext<AppState>) -> Result<Response> {
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
    mut req: Request,
    ctx: RouteContext<AppState>,
) -> Result<Response> {
    let upgrade_header = req.headers().get("Upgrade");

    if upgrade_header.is_err() || upgrade_header.unwrap().unwrap() != "websocket" {
        return Ok(Response::builder()
            .with_status(426)
            .body(ResponseBody::Empty));
    }

    if let Some(chat_id) = ctx.param("chat_id") {
        let object = ctx.durable_object("CHATROOM").unwrap();
        let id = object.id_from_name(chat_id.as_str()).unwrap();
        let stub = id.get_stub().unwrap();
        let res = stub.fetch_with_request(req.clone().unwrap()).await.unwrap();

        return Ok(res);
    }

    Response::error("Bad Request", 400)
}
