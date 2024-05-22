use crate::handlers::{
    create_image_handler, get_image_handler, CreateImageRequest, CreateImageResponse,
    GetImageRequest, GetImageResponse, GetImagesRequest, GetImagesResponse,
};
use handlers::get_images_handler;
use serde::Deserialize;
use state::ImageStore;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use worker::*;

mod handlers;
mod state;

pub struct AppState {
    image_store: ImageStore,
}

#[derive(Deserialize)]
pub struct GetImagesQueryParameters {
    count: Option<usize>,
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let _subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    let database = env.d1("DB").unwrap();
    let service_binding = env.service("AUTH").unwrap();

    let auth_response = service_binding.fetch_request(req.clone().unwrap()).await.unwrap();

    if auth_response.status_code() != 200 {
        return Ok(auth_response);
    }

    Router::with_data(AppState {
        image_store: ImageStore::new(database),
    })
    .get_async("/images", get_images)
    .get_async("/images/:id", get_image)
    .post_async("/images", create_image)
    .or_else_any_method("/*catchall", not_found)
    .run(req, env)
    .await
}

pub async fn create_image(mut req: Request, ctx: RouteContext<AppState>) -> Result<Response> {
    let request_body: Result<CreateImageRequest> = req.json().await;

    let result = match request_body {
        Ok(parsed_body) => {
            let result = create_image_handler(parsed_body, ctx.data.image_store).await;

            Response::from_json(&result).unwrap()
        }
        Err(_) => Response::from_json(&CreateImageResponse {
            message: "Failure parsing request body".to_string(),
            image: None,
        })
        .unwrap()
        .with_status(400),
    };

    Ok(result)
}

pub async fn get_images(req: Request, ctx: RouteContext<AppState>) -> Result<Response> {
    let count: GetImagesQueryParameters = req
        .query()
        .unwrap_or(GetImagesQueryParameters { count: Some(10) });

    let result = get_images_handler(
        GetImagesRequest {
            count: count.count.unwrap_or(10),
        },
        &ctx.data.image_store,
    )
    .await;

    match result {
        Ok(images) => Response::from_json(&images),
        Err(_) => Response::from_json(&GetImagesResponse {
            images: None,
            message: "OK".to_string(),
        }),
    }
}

pub async fn get_image(_req: Request, ctx: RouteContext<AppState>) -> Result<Response> {
    let id_param = ctx.param("id").unwrap();

    match id_param.parse::<i32>() {
        Ok(id) => {
            let result = get_image_handler(GetImageRequest { id }, &ctx.data.image_store).await;

            let response = match result {
                Ok(img) => Response::from_json(&img).unwrap(),
                Err(img) => Response::from_json(&img).unwrap().with_status(404),
            };

            Ok(response)
        }
        Err(_) => Ok(Response::from_json(&GetImageResponse {
            image: None,
            message: "Invalid id".to_string(),
        })
        .unwrap()
        .with_status(400)),
    }
}

pub fn not_found(_req: Request, _ctx: RouteContext<AppState>) -> Result<Response> {
    let mut req_headers = Headers::new();
    req_headers.set("Content-Type", "application/json")?;
    Ok(ResponseBuilder::new()
        .with_headers(req_headers)
        .with_status(404)
        .body(ResponseBody::Body("{}".to_string().into_bytes())))
}

#[cfg(test)]
mod tests {
    use crate::state::Image;

    #[tokio::test]
    async fn test_list_images_should_return() -> Result<(), Box<dyn std::error::Error>> {
        let resp = reqwest::get("http://localhost:8787/images").await?;

        assert_eq!(resp.status(), 200);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_image_should_return() -> Result<(), Box<dyn std::error::Error>> {
        let resp = reqwest::get("http://localhost:8787/images/1").await?;

        assert_eq!(resp.status(), 200);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_image_with_invalid_path_should_return_400(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let resp = reqwest::get("http://localhost:8787/images/iowefnwoiefg").await?;

        assert_eq!(resp.status(), 400);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_image_with_unknown_id_should_return_404(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let resp = reqwest::get("http://localhost:8787/images/987").await?;

        assert_eq!(resp.status(), 404);

        Ok(())
    }

    #[tokio::test]
    async fn test_404_response_for_invalid_endpoint() -> Result<(), Box<dyn std::error::Error>> {
        let resp = reqwest::get("http://localhost:8787/this-is-an-unknown-endpoint").await?;

        assert_eq!(resp.status(), 404);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_image_should_return() -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let resp = client
            .post("http://localhost:8787/images")
            .body(
                serde_json::to_string(&Image::new(
                    1,
                    1,
                    "https://test.com".to_string(),
                    "Test".to_string(),
                    "png".to_string(),
                    "600x600".to_string(),
                    100,
                ))
                .unwrap(),
            )
            .send()
            .await?;

        assert_eq!(resp.status(), 200);

        Ok(())
    }
}

// #[event(scheduled)]
// async fn scheduled(req: ScheduledEvent, env: Env, _ctx: ScheduleContext) -> () {
//     let subscriber = FmtSubscriber::builder()
//         // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
//         // will be written to stdout.
//         .with_max_level(Level::TRACE)
//         // completes the builder.
//         .finish();

//     tracing::subscriber::set_global_default(subscriber)
//         .expect("setting default subscriber failed");

//     console_error_panic_hook::set_once();

//     match req.cron().as_str() {
//         "*/15 * * * *" => info!("Hello from a 15 minute schedule"),
//         "*/30 * * * *" => info!("Hello from a 30 minute schedule"),
//         _ => info!("Unknown schedule")

//     }
// }
