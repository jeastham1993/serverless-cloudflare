use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;
use wasm_bindgen::JsValue;
use worker::D1Database;

#[derive(Deserialize)]
pub struct CreateChatCommand{
    pub name: String,
    pub password: String
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ChatDTO {
    pub id: String,
    pub name: String
}

impl ChatDTO {
    fn from(chat: &Chat) ->  Self {
        ChatDTO { id: chat.id.clone(), name: chat.name.clone() }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Chat {
    id: String,
    name: String,
    password: String,
}

impl Chat {
    pub fn new(name: String, password: String) -> Self {
        Chat {
            id: Uuid::new_v4().to_string(),
            name: name,
            password: password,
        }
    }
}

pub struct ChatRepository {
    database: D1Database,
}

impl ChatRepository {
    pub fn new(database: D1Database) -> Self {
        ChatRepository { database }
    }

    pub async fn list_all_chats(&self, limit: usize) -> Vec<ChatDTO> {
        let db_chats = &self
            .database
            .prepare(
                "SELECT id, name, password
        FROM chats c
        LIMIT ?1",
            )
            .bind(&[JsValue::from(limit)])
            .unwrap()
            .all()
            .await;

        match db_chats {
            Ok(d1_result) => d1_result.results::<Chat>().unwrap().iter().map(|chat| ChatDTO::from(chat)).collect(),
            Err(_) => Vec::new(),
        }
    }

    pub async fn get_chat(&self, id: &str) -> Result<ChatDTO, ()> {
        let db_chats = &self
            .database
            .prepare(
                "SELECT id, name, password
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
                Some(chat) => {
                    Ok(ChatDTO::from(chat))
                }
            },
            Err(_) => Err(()),
        }
    }

    pub async fn validate_password(&self, id: &str, password: &str) -> bool {
        let db_chats = &self
            .database
            .prepare(
                "SELECT id, name, password
FROM chats c
WHERE c.id = ?1",
            )
            .bind(&[JsValue::from(id)])
            .unwrap()
            .first::<Chat>(None)
            .await;

        match db_chats {
            Ok(d1_result) => match d1_result {
                None => false,
                Some(chat) => {
                    info!("Stored password is {}", &chat.password);
                    chat.password == password
                }
            },
            Err(_) => false,
        }
    }

    pub async fn add_chat(&self, chat: Chat) -> Result<Chat, ()> {
        let insert_result = &self
            .database
            .prepare(
                "INSERT INTO chats
(id, name, password)
VALUES
(?1, ?2, ?3)
RETURNING *;",
            )
            .bind(&[
                JsValue::from(chat.id),
                JsValue::from(chat.name),
                JsValue::from(chat.password)
            ])
            .unwrap()
            .first::<Chat>(None)
            .await;

        match insert_result {
            Ok(res) => match res {
                None => Err(()),
                Some(chat) => {
                    let cloned_chat = chat.clone();

                    Ok(cloned_chat)
                }
            },
            Err(_) => Err(()),
        }
    }
}
