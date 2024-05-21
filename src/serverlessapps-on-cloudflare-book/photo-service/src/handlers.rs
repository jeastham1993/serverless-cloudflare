use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing_subscriber::filter;
use worker::{Request, Response, ResponseBody, ResponseBuilder};

use crate::state::{Image, ImageStore};

#[derive(Deserialize)]
struct GetImagesQueryParameters {
    count: usize,
}

#[derive(Serialize)]
struct GetImageResponse {
    images: Vec<Image>,
}

#[derive(Serialize)]
pub struct SingleImageResponse {
    pub message: String,
    pub image: Option<Image>,
}


pub async fn get_image_handler(id: i32, image_store: &ImageStore) -> Response {
    let image = image_store.get_image(id).await;

    match image {
        Ok(img) => Response::from_json(&SingleImageResponse{
            image: Some(img),
            message: "OK".to_string()
        }).unwrap(),
        Err(_) => Response::from_json(&SingleImageResponse{
            image: None,
            message: "Image not Found".to_string()
        }).unwrap().with_status(404)
    }
}

pub async fn get_images_handler(request: Request, image_store: &ImageStore) -> Response {
    let count = request.query::<GetImagesQueryParameters>();

    let count = match count {
        Ok(count_value) => count_value.count,
        Err(_) => 100,
    };

    let images = image_store.get_images(count).await;

    Response::from_json(&GetImageResponse { images }).unwrap()
}

pub async fn create_image_handler(mut request: Request, mut image_store: ImageStore) -> Response {
    let request_body = request.json().await;

    let image: Image = match request_body {
        Ok(body) => body,
        Err(_) => {
            return Response::from_json(&SingleImageResponse { image: None, message: "Invalid image in POST body".to_string() }).unwrap().with_status(400)
        }
    };

    let created_img = image_store.add_image(image.clone()).await;

    match created_img {
        Ok(img) => Response::from_json(&SingleImageResponse { image: Some(img), message: "OK".to_string() }).unwrap().with_status(200),
        Err(_) => return Response::from_json(&SingleImageResponse { image: None, message: "Failure saving image".to_string() }).unwrap().with_status(400)
    }

}
