use serde::{Deserialize, Serialize};
use tokio_postgres::Client;
use uuid::Uuid;
use thiserror::Error;

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
    client: Client,
}

impl ToDoRepository {
    pub fn new(client: Client) -> Self {
        Self{
            client,
        }
    }

    pub async fn list(&self, username: String) -> Result<Vec<ToDo>, RepositoryError> {
        let rows = &self.client
            .query("SELECT id, name, username FROM todos WHERE username = $1::TEXT;", &[&username])
            .await
            .map_err(|err| {
                RepositoryError::UnknownError("Unknown failure querying database".to_string())
            })?;

        let mut results = Vec::new();

        for row in rows {
            let id: String = row.get("id");
            let user_name: String = row.get("username");
            let name: String = row.get("name");
            results.push(ToDo{
                id,
                user_name,
                name
            })
        }

        Ok(results)
    }

    pub async fn add(&self, user_name: String, name: String) -> Result<String, RepositoryError> {
        let id = Uuid::new_v4().to_string();

        &self.client.execute("INSERT INTO todos (id, userName, name) VALUES ($1, $2, $3)", &[&id, &user_name, &name])
            .await
            .map_err(|err| {
                RepositoryError::UnknownError("Unknown failure querying database".to_string())
            })?;

        Ok(id)
    }

    pub async fn get(&self, id: String) -> Result<ToDo, RepositoryError> {
        let row = &self.client.query("SELECT id, name, username FROM todos WHERE id =  $1::TEXT;", &[&id])
            .await
            .map_err(|err| {
                RepositoryError::UnknownError("Unknown failure querying database".to_string())
            })?;

        let todo = ToDo{
            id: row[0].get("id"),
            name: row[0].get("name"),
            user_name:  row[0].get("username")
        };

        Ok(todo)
    }

    pub async fn delete(&self, id: String) -> Result<(), RepositoryError> {
        &self.client.execute("DELETE FROM todos WHERE id =  $1::TEXT;", &[&id])
            .await
            .map_err(|err| {
                RepositoryError::UnknownError("Unknown failure querying database".to_string())
            })?;

        Ok(())
    }
}