use serde::{Deserialize, Serialize};
use worker::*;

use crate::core::{ToDo, ToDoRepository};

mod core;

struct AppState{
    repo: ToDoRepository
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    Router::with_data(AppState{
        repo: ToDoRepository::new(env.kv("james-eastham-dev").unwrap()),
    })
        .get_async("/", list)
        .post_async("/", add)
        .run(req, env)
        .await
}

pub async fn list(req: Request, ctx: RouteContext<AppState>) -> Result<Response> {
    let res = ctx.data.repo.list("james".to_string()).await;

    match res {
        Ok(data) => Response::from_json(&ListToDoApiResponse{
            todos: data
        }),
        Err(_) => Response::from_json(&ListToDoApiResponse{
            todos: vec!()
        })
    }
}

pub async fn add(mut req: Request, ctx: RouteContext<AppState>) -> Result<Response> {
    let body: AddToDoRequest = req.json().await.unwrap();
    let res = ctx.data.repo.add(body.user_name, body.name).await;

    match res {
        Ok(created_id) => Response::from_json(&AddToDoApiResponse{
            created_id
        }),
        Err(_) => Response::from_json(&AddToDoApiResponse{
            created_id: "".to_string()
        })
    }
}

// pub async fn list(State(repo): State<Arc<ToDoRepository>>) -> (StatusCode, Json<ListToDoApiResponse>) {
//     let res = repo.list("james".to_string()).await;
//
//     match res {
//         Ok(data) => (StatusCode::OK, Json(ListToDoApiResponse{
//             todos: data
//         })),
//         Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ListToDoApiResponse{
//             todos: vec!()
//         }))
//     }
// }
//
// pub async fn add(State(repo): State<Arc<ToDoRepository>>, Json(req): Json<AddToDoRequest>) -> (StatusCode, Json<AddToDoApiResponse>) {
//     let res = repo.add(req.user_name, req.name).await;
//
//     match res {
//         Ok(created_id) => (StatusCode::OK, Json(AddToDoApiResponse{
//             created_id
//         })),
//         Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(AddToDoApiResponse{
//             created_id: "".to_string()
//         }))
//     }
// }
//
// pub async fn delete(State(repo): State<Arc<ToDoRepository>>, Path(id): Path<String>) -> (StatusCode, Json<DeleteToDoResponse>) {
//     let res = repo.delete(id).await;
//
//     match res {
//         Ok(created_id) => (StatusCode::OK, Json(DeleteToDoResponse{
//         })),
//         Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(DeleteToDoResponse{
//         }))
//     }
// }
//
// pub async fn get_todo(State(repo): State<Arc<ToDoRepository>>, Path(id): Path<String>) -> (StatusCode, Json<DeleteToDoResponse>) {
//     let res = repo.get(id).await;
//
//     match res {
//         Ok(created_id) => (StatusCode::OK, Json(DeleteToDoResponse{
//         })),
//         Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(DeleteToDoResponse{
//         }))
//     }
// }

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