use serde::{Deserialize, Serialize};
use tracing::info;
use worker::{durable_object, Env, Request, Response, Result, State, WebSocketPair};

#[derive(Deserialize, Serialize, Clone)]
pub struct Message {
    contents: String,
    user: String
}

#[durable_object]
pub struct Chatroom {
    message_history: Vec<Message>,
    state: State,
    env: Env,
}

#[durable_object]
impl DurableObject for Chatroom {
    fn new(state: State, env: Env) -> Self {
        Self {
            message_history: vec![],
            state: state,
            env,
        }
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        let url = req.url().unwrap();

        let paths = url.path_segments().unwrap();

        let paths = paths.collect::<Box<[_]>>();

        if paths.first() == Some(&"message") {
            match req.method() {
                worker::Method::Get => self.load_message_history(req).await,
                worker::Method::Post => self.new_message(req).await,
                _ => Ok(Response::builder()
                    .with_status(404)
                    .body(worker::ResponseBody::Empty)),
            }
        } else if paths.first() == Some(&"connect") {
            info!("Connecting websocket");
            let WebSocketPair { client, server } = WebSocketPair::new()?;
            self.state.accept_web_socket(&server);

            Ok(Response::from_websocket(client)?)
        } else {
            Response::from_body(worker::ResponseBody::Empty)
        }
    }
}

impl Chatroom {
    async fn load_message_history(&mut self, mut req: Request) -> Result<Response> {
        let stored_messages = self.state.storage().get::<Vec<Message>>("messages").await;

        let messages = match stored_messages {
            Ok(messages) => messages,
            Err(_) => vec![],
        };

        Response::from_json(&messages)
    }

    async fn new_message(&mut self, mut req: Request) -> Result<Response> {
        let stored_messages = self.state.storage().get::<Vec<Message>>("messages").await;

        let mut messages = match stored_messages {
            Ok(messages) => messages,
            Err(_) => vec![],
        };

        let message: Message = req.json().await.unwrap();

        messages.push(message.clone());

        self.state.storage().put("messages", &messages).await;

        let web_socket_conns = self.state.get_websockets();

        for conn in web_socket_conns {
            let _ = conn.send(&message);
        }

        Response::from_json(&messages)
    }
}