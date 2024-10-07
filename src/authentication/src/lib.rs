use std::sync::Arc;

use auth::AuthenticationService;
use serde::{Deserialize, Serialize};
use tokio_postgres::{Error, NoTls};
use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime},
    prelude::*,
};
use tracing_web::{performance_layer, MakeConsoleWriter};
use users::{LoginCommand, RegisterCommand, UserRepository};
use worker::{postgres_tls::PassthroughTls, *};

mod auth;
mod users;
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    pub sub: String,
    pub exp: usize,
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
    user_repository: UserRepository,
    auth_service: AuthenticationService,
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    let jwt_secret = env.secret("JWT_SECRET")?.to_string();

    let user_notifications_queue = env.queue("USER_NOTIFICATIONS")?;

    let hyperdrive = env.hyperdrive("DB")?;

    let config = hyperdrive
        .connection_string()
        .parse::<tokio_postgres::Config>()
        .map_err(|e| worker::Error::RustError(format!("Failed to parse configuration: {:?}", e)))?;

    let host = hyperdrive.host();
    let port = hyperdrive.port();

    let socket = Socket::builder()
        .secure_transport(SecureTransport::StartTls)
        .connect(host, port)?;

    let (client, connection) = config
        .connect_raw(socket, PassthroughTls)
        .await
        .map_err(|e| worker::Error::RustError(format!("Failed to connect: {:?}", e)))?;

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(error) = connection.await {
            console_log!("connection error: {:?}", error);
        }
    });

    Router::with_data(AppState {
        user_repository: UserRepository::new(client, user_notifications_queue),
        auth_service: AuthenticationService::new(jwt_secret),
    })
    .post_async("/api/register", handle_register)
    .post_async("/api/login", handle_login)
    .run(req, env)
    .await
}

pub async fn handle_register(mut req: Request, ctx: RouteContext<AppState>) -> Result<Response> {
    tracing::info!("Handle register");

    let command: RegisterCommand = req.json().await?;

    match command.handle(&ctx.data.user_repository).await {
        Ok(user_result) => Response::from_json(&user_result),
        Err(e) => match e {
            users::UserErrors::UnknownFailure => Response::error("Failed to register user", 500),
            users::UserErrors::Exists => Response::error("User exists ", 400),
            users::UserErrors::InvalidPassword => Response::error("Failed to register user", 500),
        },
    }
}

pub async fn handle_login(mut req: Request, ctx: RouteContext<AppState>) -> Result<Response> {
    let command: LoginCommand = req.json().await?;

    match command
        .handle(&ctx.data.user_repository, &ctx.data.auth_service)
        .await
    {
        Ok(resp) => Response::from_json(&resp),
        Err(e) => match e {
            users::UserErrors::InvalidPassword => Response::error("Unauthorized", 401),
            users::UserErrors::Exists => Response::error("User exists ", 400),
            users::UserErrors::UnknownFailure => Response::error("Failed ", 500),
        },
    }
}
