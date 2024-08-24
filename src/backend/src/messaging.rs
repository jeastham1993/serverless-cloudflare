use std::fmt::{Display, Formatter, Result};

use serde::{Deserialize, Serialize};

use crate::{chatroom::Chatroom, chats::Chat};

#[derive(Debug)]
pub enum MessageTypes {
    NewMessage,
    MessageHistory,
    ChatroomEnded,
    ConnectionUpdate,
}

impl Display for MessageTypes {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Deserialize)]
pub struct IncomingMessageType {
    pub message_type: String
}

#[derive(Serialize, Deserialize)]
pub struct MessageWrapper<T> {
    pub message: T,
    message_type: String,
}

impl<T> MessageWrapper<T> {
    pub fn new(message_type: MessageTypes, message: T) -> Self {
        MessageWrapper {
            message: message,
            message_type: message_type.to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Message {
    contents: String,
    user: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ChatroomEnded {
    chat_id: String
}

impl ChatroomEnded {
    pub fn new(chat_id: String) -> Self {
        ChatroomEnded{
            chat_id
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ConnectionUpdate {
    connection_count: i32,
    online_users: Vec<String>
}

impl ConnectionUpdate{
    pub fn new(connection_count: i32, online_users: Vec<String>) -> Self {
        ConnectionUpdate{
            connection_count,
            online_users
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MessageHistory {
    history: Vec<Message>
}

impl MessageHistory {
    pub fn new(history: Vec<Message>) -> Self {
        MessageHistory {
            history
        }
    }
}