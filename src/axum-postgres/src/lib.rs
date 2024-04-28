use std::sync::Arc;

use axum::{extract::{State, Path}, Json, Router, routing::get};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use serde::{Deserialize, Serialize};
use tokio_postgres::Client;
use tokio_postgres::config::SslMode;
use tower_service::Service;
use worker::*;
use worker::postgres_tls::PassthroughTls;

use crate::core::{ToDo, ToDoRepository};

mod core;

fn router(client: Client) -> Router {
    let repository = Arc::new(ToDoRepository::new(client));

    Router::new()
        .route("/", get(list))
        .route("/", post(add))
        .route("/:id", post(get_todo))
        .route("/:id", post(delete))
        .with_state(repository)
}

#[event(fetch)]
async fn fetch(req: HttpRequest,
               env: Env,
               _ctx: Context,) -> Result<axum::http::Response<axum::body::Body>> {
    let mut config = tokio_postgres::config::Config::new();

    // Configure username, password, database as needed.
    config.user(env.var("DB_USER").unwrap().to_string().as_str());
    config.password(env.var("DB_PASS").unwrap().to_string());
    config.dbname(env.var("DB_NAME").unwrap().to_string().as_str());

    config.ssl_mode(SslMode::Require);

    let socket = Socket::builder()
        .secure_transport(SecureTransport::StartTls)
        .connect(env.var("DB_HOST").unwrap().to_string().as_str(), 5432)?;
    let (client, connection) = config
        .connect_raw(socket, PassthroughTls)
        .await
        .map_err(|e| worker::Error::RustError(format!("tokio-postgres: {:?}", e)))?;

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(error) = connection.await {
            console_log!("connection error: {:?}", error);
        }
    });

    Ok(router(client).call(req).await?)
}


pub async fn list(State(repo): State<Arc<ToDoRepository>>) -> (StatusCode, Json<ListToDoApiResponse>) {
    let res = repo.list("james".to_string()).await;

    match res {
        Ok(data) => (StatusCode::OK, Json(ListToDoApiResponse{
            todos: data
        })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ListToDoApiResponse{
            todos: vec!()
        }))
    }
}

pub async fn add(State(repo): State<Arc<ToDoRepository>>, Json(req): Json<AddToDoRequest>) -> (StatusCode, Json<AddToDoApiResponse>) {
    let res = repo.add(req.user_name, req.name).await;

    match res {
        Ok(created_id) => (StatusCode::OK, Json(AddToDoApiResponse{
            created_id
        })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(AddToDoApiResponse{
            created_id: "".to_string()
        }))
    }
}

pub async fn delete(State(repo): State<Arc<ToDoRepository>>, Path(id): Path<String>) -> (StatusCode, Json<DeleteToDoResponse>) {
    let res = repo.delete(id).await;

    match res {
        Ok(created_id) => (StatusCode::OK, Json(DeleteToDoResponse{
        })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(DeleteToDoResponse{
        }))
    }
}

pub async fn get_todo(State(repo): State<Arc<ToDoRepository>>, Path(id): Path<String>) -> (StatusCode, Json<DeleteToDoResponse>) {
    let res = repo.get(id).await;

    match res {
        Ok(created_id) => (StatusCode::OK, Json(DeleteToDoResponse{
        })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(DeleteToDoResponse{
        }))
    }
}

#[derive(Serialize, Deserialize)]
struct ListToDoApiResponse{
    todos: Vec<ToDo>
}

#[derive(Deserialize)]
struct AddToDoRequest{
    user_name: String,
    name: String
}

#[derive(Serialize, Deserialize)]
struct AddToDoApiResponse{
    created_id: String
}

#[derive(Serialize, Deserialize)]
struct DeleteToDoResponse{
}

#[derive(Debug, Deserialize, Serialize)]
struct GenericResponse {
    status: u16,
    message: String,
}