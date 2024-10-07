use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;
use wasm_bindgen::JsValue;
use worker::{kv::KvStore, D1Database};

#[derive(Deserialize)]
pub struct CreateChatCommand {
    pub name: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ChatDTO {
    pub id: String,
    pub name: String,
}

impl ChatDTO {
    fn from(chat: &Chat) -> Self {
        ChatDTO {
            id: chat.id.clone(),
            name: chat.name.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Chat {
    pub id: String,
    pub name: String,
    pub created_by: String,
}

impl Chat {
    pub fn new(name: String, created_by: String) -> Self {
        Chat {
            id: Uuid::new_v4().to_string(),
            name,
            created_by,
        }
    }
}

pub struct ChatRepository {
    database: D1Database,
    cache: KvStore,
}

impl ChatRepository {
    pub fn new(database: D1Database, kv_store: KvStore) -> Self {
        ChatRepository {
            database,
            cache: kv_store,
        }
    }

    async fn list_from_db(&self, limit: usize) -> Vec<ChatDTO> {
        info!("Cache miss");
        let db_chats = &self
            .database
            .prepare(
                "SELECT id, name, created_by
FROM chats c
LIMIT ?1",
            )
            .bind(&[JsValue::from(limit)])
            .unwrap()
            .all()
            .await;

        match db_chats {
            Ok(d1_result) => {
                let db_result = d1_result
                    .results::<Chat>()
                    .unwrap()
                    .iter()
                    .map(ChatDTO::from)
                    .collect();

                let res = &self
                    .cache
                    .put("CHATS", &db_result)
                    .unwrap()
                    // TTL in workers must be at least 60 seconds
                    .expiration_ttl(60)
                    .execute()
                    .await;

                match res {
                    Ok(_) => {}
                    Err(e) => warn!("Failure writing to cache: {:?}", e),
                }

                db_result
            }
            Err(_) => Vec::new(),
        }
    }

    pub async fn list_all_chats(&self, limit: usize) -> Vec<ChatDTO> {
        let cached_chats = &self.cache.get("CHATS").json::<Vec<ChatDTO>>().await;

        match cached_chats {
            Ok(cached) => {
                info!("Cached chats found");
                match cached {
                    Some(value) => {
                        info!("Cache hit");
                        value.to_vec()
                    }
                    None => self.list_from_db(limit).await,
                }
            }
            Err(_) => self.list_from_db(limit).await,
        }
    }

    pub async fn get_chat(&self, id: &str) -> Result<ChatDTO, ()> {
        let db_chats = &self
            .database
            .prepare(
                "SELECT id, name, created_by
FROM chats c
WHERE c.id = ?1",
            )
            .bind(&[JsValue::from(id)])
            .unwrap()
            .first::<Chat>(None)
            .await;

        match db_chats {
            Ok(d1_result) => match d1_result {
                None => Err(()),
                Some(chat) => Ok(ChatDTO::from(chat)),
            },
            Err(_) => Err(()),
        }
    }

    pub async fn delete_chat(&self, chat_id: &String) -> Result<(), ()> {
        let _ = &self
            .database
            .prepare(
                "DELETE FROM chats
WHERE id = ?1",
            )
            .bind(&[JsValue::from(chat_id)])
            .unwrap()
            .run()
            .await;

        let _ = &self.cache.delete("CHATS").await;

        Ok(())
    }

    pub async fn add_chat(&self, chat: Chat) -> Result<Chat, ()> {
        let insert_result = &self
            .database
            .prepare(
                "INSERT INTO chats
            (id, name, created_by)
            VALUES
            (?1, ?2, ?3)
            RETURNING *;",
            )
            .bind(&[
                JsValue::from(chat.id),
                JsValue::from(chat.name),
                JsValue::from(chat.created_by),
            ])
            .unwrap()
            .first::<Chat>(None)
            .await;

        match insert_result {
            Ok(res) => match res {
                None => Err(()),
                Some(chat) => {
                    let _ = &self.cache.delete("CHATS").await;

                    let cloned_chat = chat.clone();
                    Ok(cloned_chat)
                }
            },
            Err(_) => Err(()),
        }
    }
}
