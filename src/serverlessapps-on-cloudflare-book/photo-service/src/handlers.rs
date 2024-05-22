use serde::{Deserialize, Serialize};

use crate::state::{Image, ImageStore};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageViewModel {
    id: i32,
}

impl From<Image> for ImageViewModel {
    fn from(value: Image) -> Self {
        ImageViewModel { id: value.get_id() }
    }
}

impl From<&Image> for ImageViewModel {
    fn from(value: &Image) -> Self {
        ImageViewModel {
            id: value.clone().get_id(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetImagesRequest {
    pub count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetImagesResponse {
    pub images: Option<Vec<ImageViewModel>>,
    pub message: String,
}

pub async fn get_images_handler(
    request: GetImagesRequest,
    image_store: &ImageStore,
) -> Result<GetImagesResponse, GetImagesResponse> {
    let images = image_store.get_images(request.count).await;

    let image_view_models = images.iter().map(ImageViewModel::from).collect();

    Ok(GetImagesResponse{
        images: Some(image_view_models),
        message: "OK".to_string()
    })
}

pub struct GetImageRequest {
    pub id: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetImageResponse {
    pub message: String,
    pub image: Option<ImageViewModel>,
}

pub async fn get_image_handler(
    req: GetImageRequest,
    image_store: &ImageStore,
) -> Result<GetImageResponse, GetImageResponse> {
    let image = image_store.get_image(req.id).await;

    match image {
        Ok(img) => Ok(GetImageResponse {
            image: Some(img.into()),
            message: "OK".to_string(),
        }),
        Err(_) => Err(GetImageResponse {
            image: None,
            message: "Failure retrieving messages".to_string(),
        }),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateImageRequest {
    category_id: i32,
    user_id: i32,
    image_url: String,
    title: String,
    format: String,
    resolution: String,
    file_size_bytes: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateImageResponse {
    pub message: String,
    pub image: Option<ImageViewModel>,
}

pub async fn create_image_handler(
    request: CreateImageRequest,
    image_store: ImageStore,
) -> Result<CreateImageResponse, CreateImageResponse> {
    let image = Image::new(
        request.category_id,
        request.user_id,
        request.image_url,
        request.title,
        request.format,
        request.resolution,
        request.file_size_bytes,
    );

    let created_img = image_store.add_image(image).await;

    match created_img {
        Ok(img) => Ok(CreateImageResponse {
            image: Some(img.into()),
            message: "Ok".to_string(),
        }),
        Err(_) => Err(CreateImageResponse {
            image: None,
            message: "Failure storing data".to_string(),
        }),
    }
}
