use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Update {
    pub update_id: i32,
    pub message: Option<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub message_id: i32,
    pub date: i32,
    pub chat: Chat,
    pub from: Option<User>,
    pub text: Option<String>,
    pub sticker: Option<Sticker>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub language_code: Option<String>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Sticker {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i32,
    pub height: i32,
    pub is_animated: bool,
    pub emoji: Option<String>,
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
    pub reply_markup: Option<ReplyKeyboardMarkup>,
}
impl ResponseMessage {
    pub fn new(method: String, chat_id: i32, text: String) -> Self {
        ResponseMessage {
            method,
            chat_id,
            text,
            reply_markup: None,
        }
    }
    pub fn keyboard(mut self, keyboard: ReplyKeyboardMarkup) -> ResponseMessage {
        self.reply_markup = Some(keyboard);
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplyKeyboardMarkup {
    pub keyboard: Vec<Vec<String>>,
    pub resize_keyboard: bool,
}
