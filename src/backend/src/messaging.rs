use std::fmt::{Display, Formatter, Result};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum MessageTypes {
    NewMessage,
    ConnectionUpdate,
}

impl Display for MessageTypes {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize)]
pub struct MessageWrapper<T> {
    message: T,
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