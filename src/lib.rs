use rocket::http::{ContentType, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Update {
    pub update_id: i32,
    pub message: Option<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub message_id: i32,
    pub from: Option<User>,
    pub date: i32,
    pub chat: Chat,
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    id: i32,
    is_bot: bool,
    first_name: String,
    last_name: Option<String>,
    username: Option<String>,
    language_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chat {
    pub id: i32,
    #[serde(rename = "type")]
    pub typ: String,
    pub title: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Entity {
    offset: i32,
    length: i32,
    #[serde(rename = "type")]
    typ: String,
}

// Send types
#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessage {
    pub chat_id: i32,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub method: String,
    pub chat_id: i32,
    pub text: String,
}
