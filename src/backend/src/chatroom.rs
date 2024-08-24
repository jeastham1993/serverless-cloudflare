use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::info;
use worker::{
    durable_object, Env, Request, Response, Result, State, WebSocket, WebSocketIncomingMessage,
    WebSocketPair,
};

use crate::{
    chats::ChatRepository,
    messaging::{ChatroomEnded, ConnectionUpdate, IncomingMessageType, Message, MessageHistory, MessageTypes, MessageWrapper},
};

#[derive(Deserialize, Serialize)]
struct QueryStringParameters {
    user_id: String,
}

#[derive(Deserialize, Serialize)]
struct WebsocketConnectionAttachements {
    user_id: String,
}

#[durable_object]
pub struct Chatroom {
    state: State,
    env: Env,
    connected_users: i32,
    user_names: Vec<String>,
    chat_repository: ChatRepository,
}

#[durable_object]
impl DurableObject for Chatroom {
    fn new(state: State, env: Env) -> Self {
        let database = env.d1("CHAT_METADATA").unwrap();

        let durable_object = Self {
            state: state,
            env,
            connected_users: 0,
            user_names: vec![],
            chat_repository: ChatRepository::new(database),
        };

        durable_object
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        let url = req.url().unwrap();

        let paths = url.path_segments().unwrap();

        let paths = paths.collect::<Box<[_]>>();

        //let _ = self.state.storage().set_alarm(Duration::from_secs(5)).await;

        match paths[1] {
            "connect" => self.handle_connect(req, paths).await,
            _ => Ok(Response::builder()
                .with_status(404)
                .body(worker::ResponseBody::Empty)),
        }
    }

    async fn alarm(&mut self) -> Result<Response> {
        info!("Alarm triggered");

        let chat_id = self.state.storage().get("chat_id").await.map_err(|e| {
            return worker::Error::RustError("Failure retrieving chat id".to_string());
        })?;

        info!("Retrieved {}", chat_id);

        let _ = self.chat_repository.delete_chat(&chat_id).await;

        let web_socket_conns = self.state.get_websockets();

        let message_wrapper =
            MessageWrapper::new(MessageTypes::ChatroomEnded, ChatroomEnded::new(chat_id));

        for conn in web_socket_conns {
            let _ = conn.send(&message_wrapper);
        }

        Response::ok("ALARMED")
    }

    async fn websocket_message(
        &mut self,
        ws: WebSocket,
        message: WebSocketIncomingMessage,
    ) -> Result<()> {
        match message {
            WebSocketIncomingMessage::String(str_data) => {
                let incoming_message: IncomingMessageType = serde_json::from_str(&str_data).unwrap();

                match incoming_message.message_type.as_str() {
                    "NewMessage" => {
                        let wrapper: MessageWrapper<Message> = serde_json::from_str(&str_data).unwrap();

                        let _ = &self.new_message(wrapper.message).await;
                    }
                    _ => ()
                }
            }
            WebSocketIncomingMessage::Binary(_) => {
                info!("Binary received");
            }
        }

        Ok(())
    }

    async fn websocket_close(
        &mut self,
        ws: WebSocket,
        _code: usize,
        _reason: String,
        _was_clean: bool,
    ) -> Result<()> {
        info!("Client disconnected");

        let connection_attachments = ws
            .deserialize_attachment::<WebsocketConnectionAttachements>()
            .map_err(|e| {
                return worker::Error::RustError("Failure parsing attachemtns".to_string());
            })?;

        let user_id = match connection_attachments {
            Some(attachments) => attachments.user_id,
            None => "".to_string(),
        };

        let _ = &self
            .update_connection_count(UpdateConnectionCountTypes::Decrease, user_id)
            .await?;

        info!("Websocket close success");

        Ok(())
    }
}

impl Chatroom {
    async fn handle_connect(&mut self, mut req: Request, paths: Box<[&str]>) -> Result<Response> {
        if let chat_id = paths[2] {
            info!("Storing chatId {}", chat_id);
            self.state.storage().put("chat_id", chat_id).await;
        }

        let user_id_query_param = req.query::<QueryStringParameters>().map_err(|e| {
            worker::Error::RustError("Failure parsing query parameters".to_string())
        })?;

        info!("Connecting websocket for {}", user_id_query_param.user_id);

        let WebSocketPair { client, server } = WebSocketPair::new()?;
        self.state.accept_web_socket(&server);

        server.serialize_attachment(&WebsocketConnectionAttachements {
            user_id: user_id_query_param.user_id.clone(),
        });

        let messages = self.load_messages().await.unwrap();

        server.send(&MessageWrapper::new(MessageTypes::MessageHistory, MessageHistory::new(messages)));

        let _ = &self
            .update_connection_count(
                UpdateConnectionCountTypes::Increase,
                user_id_query_param.user_id,
            )
            .await?;

        Ok(Response::from_websocket(client)?)
    }
    
    async fn new_message(&mut self, mut message: Message) -> Result<Response> {
        let mut messages = self.load_messages().await?;

        messages.push(message.clone());

        let store = self
            .env
            .kv("CHAT_HISTORY")
            .map_err(|e| worker::Error::RustError("Failure loading KV store".to_string()))?;

        let store_builder = store.put(&self.state.id().to_string(), &messages)?;
        let _ = store_builder.execute().await;

        let web_socket_conns = self.state.get_websockets();

        let message_wrapper = MessageWrapper::new(MessageTypes::NewMessage, message);

        for conn in web_socket_conns {
            let _ = conn.send(&message_wrapper);
        }

        Response::from_json(&messages)
    }

    async fn load_messages(&mut self) -> Result<Vec<Message>> {
        let store = self
            .env
            .kv("CHAT_HISTORY")
            .map_err(|e| worker::Error::RustError("Failure loading KV store".to_string()))?;

        let stored_messages: Option<Vec<Message>> = store
            .get(&self.state.id().to_string())
            .json()
            .await
            .map_err(|e| {
                worker::Error::RustError("Failure loading key for chatroom from store".to_string())
            })?;

        let messages = match stored_messages {
            Some(messages) => messages,
            None => vec![],
        };

        Ok(messages)
    }

    async fn update_connection_count(
        &mut self,
        change_by: UpdateConnectionCountTypes,
        username: String,
    ) -> Result<i32> {
        let current_connections = self.state.storage().get::<i32>("connected_users").await;
        let active_users = self.state.storage().get::<Vec<String>>("users").await;

        let mut connections = match current_connections {
            Ok(active_connections) => active_connections,
            Err(_) => 0,
        };

        let mut users = match active_users {
            Ok(users) => users,
            Err(_) => vec![],
        };

        connections = connections
            + match change_by {
                UpdateConnectionCountTypes::Increase => {
                    users.push(username);
                    1
                }
                UpdateConnectionCountTypes::Decrease => {
                    users.retain(|x| x != &username);
                    -1
                }
            };

        let _ = self
            .state
            .storage()
            .put("connected_users", connections)
            .await;
        let _ = self.state.storage().put("users", &users).await;

        info!("New connection count is {}", connections);

        let message_wrapper = MessageWrapper::new(
            MessageTypes::ConnectionUpdate,
            ConnectionUpdate::new(connections.clone(), users),
        );

        let web_socket_conns = self.state.get_websockets();

        for conn in web_socket_conns {
            let _ = conn.send(&message_wrapper);
        }

        Ok(connections)
    }
}

enum UpdateConnectionCountTypes {
    Increase,
    Decrease,
}
