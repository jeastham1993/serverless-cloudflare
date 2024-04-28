use serde::{Deserialize, Serialize};
use uuid::Uuid;
use thiserror::Error;
use worker::kv::KvStore;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

#[derive(Serialize, Deserialize)]
pub struct ToDo{
    id: String,
    user_name: String,
    name: String
}

pub struct ToDoRepository{
    kv: KvStore,
}

impl ToDoRepository {
    pub fn new(kv: KvStore) -> Self {
        Self{
            kv,
        }
    }

    pub async fn list(&self, username: &String) -> Result<Vec<ToDo>, RepositoryError> {
        let rows = self.kv.get(username)
            .json::<Vec<ToDo>>()
            .await
            .unwrap();

        match rows {
            None => Ok(Vec::new()),
            Some(todos) => Ok(todos)
        }
    }

    pub async fn add(self, user_name: &String, name: String) -> Result<String, RepositoryError> {
        let id = Uuid::new_v4().to_string();

        let list_res = self.list(user_name).await;

        let mut current_todos = match list_res {
            Ok(todos) => todos,
            Err(_) => vec!()
        };

        current_todos.push(ToDo{
            id: id.clone(),
            user_name: user_name.clone(),
            name
        });

        let data = serde_json::to_string(&current_todos).unwrap();

        let _ = &self.kv.put(&user_name, data).unwrap().execute().await.unwrap();

        Ok(id.clone())
    }

    pub async fn get(&self, username: &String, id: String) -> Result<ToDo, RepositoryError> {
        let user_todos = &self.list(username).await?;

        match user_todos.iter().find(|todo| todo.id == id) {
            Some(todo) => Ok(ToDo{
                id: todo.id.clone(),
                name: todo.name.clone(),
                user_name: todo.user_name.clone()
            }),
            None => Err(RepositoryError::UnknownError("Todo not found".to_string()))
        }
    }

    pub async fn delete(&self, username: &String, id: String) -> Result<(), RepositoryError> {
        let mut user_todos = self.list(username).await?;

        let index = user_todos.iter().position(|todo| todo.id == id);
        if let Some(index) = index {
            user_todos.remove(index);
        } else {
            return Err(RepositoryError::UnknownError("Todo not found".to_string()));
        }

        let data = serde_json::to_string(&user_todos).unwrap();

        let _ = &self.kv.put(&username, data).unwrap().execute().await.unwrap();

        Ok(())
    }
}