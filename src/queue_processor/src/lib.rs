use sendgrid::{Destination, Mail, SGClient};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use tracing_web::{performance_layer, MakeConsoleWriter};
use worker::*;

#[derive(Deserialize, Serialize, Debug)]
pub struct UserDTO {
    username: String,
    email_address: String,
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

#[event(queue)]
pub async fn main(message_batch: MessageBatch<UserDTO>, env: Env, _: Context) -> Result<()> {
    console_error_panic_hook::set_once();

    let api_key = env.secret("EMAIlL_API_KEY")?.to_string();
    let from_address = env.secret("FROM_ADDRESS")?.to_string();

    let sg = SGClient::new(api_key);

    let mut x_smtpapi = String::new();
    x_smtpapi.push_str(r#"{"unique_args":{"test":7}}"#);

    for message in message_batch.messages()? {
        tracing::info!(
            "Got message {:?}, with id {} and timestamp: {}",
            message.body(),
            message.id(),
            message.timestamp().to_string(),
        );
        tracing::info!("Username is {:?}", message.body().username);
        tracing::info!("Email is {:?}", message.body().email_address);

        let email_body = format!("<h1>Thankyou for joining us!</h1><p>Hey {},</p><p>It's great to have you on this journey into the wonderful world of serverless technologies, Rust and Cloudflare</p><p>This message is coming to you all the way from a Cloudflare queue. Cool, right?</p><p>Thanks again,</p><p>James</p>", &message.body().username);

        let mail_info = Mail::new()
            .add_to(Destination {
                address: &message.body().email_address,
                name: &message.body().username,
            })
            .add_from(&from_address)
            .add_subject("Welcome to Serverless Chats")
            .add_html(&email_body)
            .add_x_smtpapi(&x_smtpapi);

        match sg.send(mail_info).await {
            Err(err) => tracing::info!("Error: {}", err),
            Ok(body) => tracing::info!("Response: {:?}", body),
        };

        message.ack();
    }

    Ok(())
}

pub struct AppState {
    queue: Queue,
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    let user_notifications_queue = env.queue("USER_NOTIFICATIONS")?;

    Router::with_data(AppState {
        queue: user_notifications_queue,
    })
    .get_async("/api/test", handle_test)
    .run(req, env)
    .await
}

pub async fn handle_test(req: Request, ctx: RouteContext<AppState>) -> Result<Response> {
    tracing::info!("Handle register");

    let query_string = req.query::<QueryStringParameters>();

    match query_string {
        Ok(query) => {
            let _sendres = ctx
                .data
                .queue
                .send(UserDTO {
                    username: query.username,
                    email_address: query.email_address,
                })
                .await;

            Response::from_html("<p>Success</p>")
        }
        Err(_) => {
            Response::from_html("<p>Failure, please pass 'email_address' and 'username' query string parameters</p>")
        },
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct QueryStringParameters {
    email_address: String,
    username: String,
}
