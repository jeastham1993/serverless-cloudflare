use serde::{Deserialize, Serialize};
use tracing_subscriber::{
    fmt::{format::Pretty, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use tracing_web::{performance_layer, MakeConsoleWriter};
use wasm_bindgen::JsValue;
use worker::*;

#[derive(Deserialize, Debug)]
pub struct UserDTO {
    username: String,
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
pub async fn main(message_batch: MessageBatch<UserDTO>, _: Env, _: Context) -> Result<()> {
    console_error_panic_hook::set_once();

    for message in message_batch.messages()? {
        tracing::info!(
            "Got message {:?}, with id {} and timestamp: {}",
            message.body(),
            message.id(),
            message.timestamp().to_string(),
        );
        tracing::info!("Username is {:?}", message.body().username);

        message.ack();
    }

    Ok(())
}
