use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use worker::*;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let _subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    let api_key_header = req.headers().get("x-api-auth-key");

    Ok(match api_key_header{
        Ok(api_key) => match api_key {
            None => ResponseBuilder::new().with_status(401).body(ResponseBody::Body(String::from("Unauthenticated").into_bytes())),
            Some(key) => {
                if key == env.var("API_AUTH_KEY").unwrap().to_string(){
                    Response::ok("Authenticated").unwrap()
                }
                else {
                    ResponseBuilder::new().with_status(401).body(ResponseBody::Body(String::from("Unauthenticated").into_bytes()))
                }
            }
        },
        Err(_) => ResponseBuilder::new().with_status(401).body(ResponseBody::Body(String::from("Unauthenticated").into_bytes()))
    })
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn unauthenticated_request_should_return_401() -> Result<(), Box<dyn std::error::Error>> {
        let resp = reqwest::get("http://localhost:8787").await?;

        assert_eq!(resp.status(), 401);

        Ok(())
    }

    #[tokio::test]
    async fn authenticated_request_should_return_200() -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let resp = client.get("http://localhost:8787")
            .header("x-api-auth-key", "mysupersecretkey")
            .send().await?;

        assert_eq!(resp.status(), 200);

        Ok(())
    }
}