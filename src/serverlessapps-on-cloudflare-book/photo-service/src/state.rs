use serde::{Deserialize, Serialize};
use worker::wasm_bindgen::JsValue;
use worker::D1Database;

#[derive(Serialize, Deserialize, Clone)]
pub struct Image {
    id: i32,
    category_id: i32,
    user_id: i32,
    image_url: String,
    title: String,
    format: String,
    resolution: String,
    file_size_bytes: i32,
}

impl Image {
    pub fn new(
        category_id: i32,
        user_id: i32,
        image_url: String,
        title: String,
        format: String,
        resolution: String,
        file_size_bytes: i32,
    ) -> Self {
        Self {
            id: -1,
            category_id,
            user_id,
            image_url,
            title,
            format,
            resolution,
            file_size_bytes,
        }
    }

    pub fn get_id(self) -> i32 {
        self.id.clone()
    }
}

pub struct ImageStore {
    database: D1Database,
}

impl ImageStore {
    pub fn new(database: D1Database) -> Self {
        ImageStore { database }
    }

    pub async fn get_images(&self, limit: usize) -> Vec<Image> {
        let db_images = &self
            .database
            .prepare(
                "SELECT i.*, c.display_name AS category_display_name
            FROM images i
            INNER JOIN image_categories c ON i.category_id = c.id
            ORDER BY created_at DESC
            LIMIT ?1",
            )
            .bind(&[JsValue::from(limit)])
            .unwrap()
            .all()
            .await;

        match db_images {
            Ok(d1_result) => d1_result.results().unwrap(),
            Err(_) => Vec::new(),
        }
    }

    pub async fn get_image(&self, id: i32) -> Result<Image, ()> {
        let db_images = &self
            .database
            .prepare(
                "SELECT i.*, c.display_name AS category_display_name
FROM images i
INNER JOIN image_categories c ON i.category_id = c.id
WHERE i.id = ?1",
            )
            .bind(&[JsValue::from(id)])
            .unwrap()
            .first::<Image>(None)
            .await;

        match db_images {
            Ok(d1_result) => match d1_result {
                None => Err(()),
                Some(img) => {
                    let new_image = img.clone();

                    Ok(new_image)
                }
            },
            Err(_) => Err(()),
        }
    }

    pub async fn add_image(&self, image: Image) -> Result<Image, ()> {
        let insert_result = &self
            .database
            .prepare(
                "INSERT INTO images
(category_id, user_id, image_url, title,
format, resolution, file_size_bytes)
VALUES
(?1, ?2, ?3, ?4, ?5, ?6, ?7)
RETURNING *;",
            )
            .bind(&[
                JsValue::from(image.category_id),
                JsValue::from(image.user_id),
                JsValue::from(image.image_url),
                JsValue::from(image.title),
                JsValue::from(image.format),
                JsValue::from(image.resolution),
                JsValue::from(image.file_size_bytes),
            ])
            .unwrap()
            .first::<Image>(None)
            .await;

        match insert_result {
            Ok(res) => match res {
                None => Err(()),
                Some(img) => {
                    let cloned_image = img.clone();

                    Ok(cloned_image)
                }
            },
            Err(_) => Err(()),
        }
    }
}
