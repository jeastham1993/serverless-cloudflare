use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use worker::{
    durable_object, Env, Request, Response, Result, State, WebSocket, WebSocketIncomingMessage,
    WebSocketPair,
};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

use crate::{
    chats::ChatRepository,
    messaging::{
        ChatroomEnded, ConnectionUpdate, IncomingMessageType, Message, MessageHistory,
        MessageTypes, MessageWrapper,
    },
};

#[derive(Deserialize, Serialize)]
struct QueryStringParameters {
    user_id: String,
}

#[derive(Deserialize, Serialize)]
struct WebsocketConnectionAttachments {
    user_id: String,
}

#[durable_object]
pub struct Chatroom {
    state: State,
    _env: Env,
    chat_repository: ChatRepository,
    messages_storage_key: String,
    chat_expiry_in_seconds: u64,
}

#[durable_object]
impl DurableObject for Chatroom {
    fn new(state: State, env: Env) -> Self {
        let database = env.d1("CHAT_METADATA").unwrap();
        let cache = env.kv("CHAT_CACHE").unwrap();

        Self {
            state,
            _env: env,
            chat_repository: ChatRepository::new(database, cache),
            messages_storage_key: "messages".to_string(),
            chat_expiry_in_seconds: 300,
        }
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        let url = req.url().unwrap();

        let paths = url.path_segments().unwrap();

        let paths = paths.collect::<Box<[_]>>();

        let _ = &self.update_chat_expiry().await;

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
            warn!("{}", e);
            worker::Error::RustError("Failure retrieving chat id".to_string())
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
        _ws: WebSocket,
        message: WebSocketIncomingMessage,
    ) -> Result<()> {
        let _ = self.update_chat_expiry().await;

        match message {
            WebSocketIncomingMessage::String(str_data) => {
                let incoming_message: IncomingMessageType =
                    serde_json::from_str(&str_data).unwrap();

                if incoming_message.message_type.as_str() == "NewMessage" {
                    let wrapper: MessageWrapper<Message> = serde_json::from_str(&str_data).unwrap();

                    let _ = &self.new_message(wrapper.message).await;
                }
            }
            WebSocketIncomingMessage::Binary(binary_data) => {
                let incoming_message: IncomingMessageType =
                    serde_json::from_slice(&binary_data).unwrap();

                if incoming_message.message_type.as_str() == "NewMessage" {
                    let wrapper: MessageWrapper<Message> =
                        serde_json::from_slice(&binary_data).unwrap();

                    let _ = &self.new_message(wrapper.message).await;
                }
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
            .deserialize_attachment::<WebsocketConnectionAttachments>()
            .map_err(|e| {
                warn!("{}", e);
                worker::Error::RustError("Failure parsing attachments".to_string())
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
    async fn update_chat_expiry(&mut self) -> () {
        // Chats are only active for a rolling 5 minute window.
        let _ = self
            .state
            .storage()
            .set_alarm(Duration::from_secs(self.chat_expiry_in_seconds.clone()))
            .await;
    }

    async fn handle_connect(&mut self, req: Request, paths: Box<[&str]>) -> Result<Response> {
        let chat_id = paths[2];

        info!("Storing chatId {}", chat_id);
        self.state
            .storage()
            .put("chat_id", chat_id)
            .await
            .map_err(|e| {
                warn!("{}", e);
                worker::Error::RustError("Failure updating chat_id against DO storage".to_string())
            })?;

        let user_id_query_param = req.query::<QueryStringParameters>().map_err(|e| {
            warn!("{}", e);
            worker::Error::RustError("Failure parsing query parameters".to_string())
        })?;

        info!("Connecting websocket for {}", user_id_query_param.user_id);

        let WebSocketPair { client, server } = WebSocketPair::new()?;
        self.state.accept_web_socket(&server);

        server
            .serialize_attachment(&WebsocketConnectionAttachments {
                user_id: user_id_query_param.user_id.clone(),
            })
            .map_err(|e| {
                warn!("{}", e);
                worker::Error::RustError(
                    "Failure adding attachment to websocket connection".to_string(),
                )
            })?;

        let messages = self.load_messages().await.map_err(|e| {
            warn!("{}", e);
            worker::Error::RustError("Failure loading messages from datastore".to_string())
        })?;

        server
            .send(&MessageWrapper::new(
                MessageTypes::MessageHistory,
                MessageHistory::new(messages),
            ))
            .unwrap_or(());

        let _ = &self
            .update_connection_count(
                UpdateConnectionCountTypes::Increase,
                user_id_query_param.user_id,
            )
            .await?;

        Response::from_websocket(client)
    }

    async fn new_message(&mut self, message: Message) -> Result<Response> {
        let mut messages = self.load_messages().await?;

        messages.push(message.clone());

        // Consider limiting the number of stored messages
        if messages.len() > 100 {
            messages = messages.split_off(messages.len() - 100);
        }

        self.state
            .storage()
            .put(&self.messages_storage_key, &messages)
            .await
            .map_err(|e| {
                warn!("{}", e);
                worker::Error::RustError("Failuring updating messages in DO storage".to_string())
            })?;

        let web_socket_conns = self.state.get_websockets();

        let message_wrapper = MessageWrapper::new(MessageTypes::NewMessage, message);

        for conn in web_socket_conns {
            let _ = conn.send(&message_wrapper);
        }

        Response::from_json(&messages)
    }

    async fn load_messages(&mut self) -> Result<Vec<Message>> {
        match self.state.storage().get::<Vec<Message>>(&self.messages_storage_key).await {
            Ok(messages) => {
                info!("Stored message count {}", messages.len());
                Ok(messages)
            },
            Err(e) => {
                warn!("Error loading messages: {}", e);
                let messages = Vec::new();
                self.state.storage().put(&self.messages_storage_key, &messages).await?;
                Ok(messages)
            }
        }
    }

    async fn update_connection_count(
        &mut self,
        change_by: UpdateConnectionCountTypes,
        username: String,
    ) -> Result<i32> {
        let current_connections = self.state.storage().get::<i32>("connected_users").await;
        let active_users = self.state.storage().get::<Vec<String>>("users").await;

        let mut connections = current_connections.unwrap_or(0);

        let mut users = match active_users {
            Ok(users) => users,
            Err(_) => vec![],
        };

        connections += match change_by {
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
            ConnectionUpdate::new(connections, users),
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
