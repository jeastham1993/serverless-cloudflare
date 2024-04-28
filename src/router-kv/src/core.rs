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

    pub async fn list(&self, username: String) -> Result<Vec<ToDo>, RepositoryError> {
        let rows = self.kv.get(&username)
            .json::<Vec<ToDo>>()
            .await
            .unwrap();

        match rows {
            None => Ok(Vec::new()),
            Some(todos) => Ok(todos)
        }
    }

    pub async fn add(self, user_name: String, name: String) -> Result<String, RepositoryError> {
        let id = Uuid::new_v4().to_string();

        let list_res = self.list(user_name.clone()).await;

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

    // pub async fn get(&self, id: String) -> Result<ToDo, RepositoryError> {
    //     let row = &self.client.query("SELECT id, name, username FROM todos WHERE id =  $1::TEXT;", &[&id])
    //         .await
    //         .map_err(|err| {
    //             RepositoryError::UnknownError("Unknown failure querying database".to_string())
    //         })?;
    //
    //     let todo = ToDo{
    //         id: row[0].get("id"),
    //         name: row[0].get("name"),
    //         user_name:  row[0].get("username")
    //     };
    //
    //     Ok(todo)
    // }
    //
    // pub async fn delete(&self, id: String) -> Result<(), RepositoryError> {
    //     &self.client.execute("DELETE FROM todos WHERE id =  $1::TEXT;", &[&id])
    //         .await
    //         .map_err(|err| {
    //             RepositoryError::UnknownError("Unknown failure querying database".to_string())
    //         })?;
    //
    //     Ok(())
    // }
}