use crate::bot_types::Payload;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Update {
    pub update_id: i32,
    pub message: Option<Message>,
    pub pre_checkout_query: Option<PreCheckoutQuery>,
    pub successful_payment: Option<SuccessfulPayment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub message_id: i32,
    pub date: i32,
    pub chat: Chat,
    pub from: Option<User>,
    pub text: Option<String>,
    pub sticker: Option<Sticker>,
    pub successful_payment: Option<SuccessfulPayment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreCheckoutQuery {
    pub id: String,
    pub from: User,
    pub currency: String,
    // In cents (1.50EUR = 150)
    pub total_amount: i32,
    pub invoice_payload: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessfulPayment {
    pub currency: String,
    // In cents (1.50EUR = 150)
    pub total_amount: i32,
    pub invoice_payload: String,
    pub telegram_payment_charge_id: String,
    pub provider_payment_charge_id: String,
}
impl SuccessfulPayment {
    pub fn get_payload(&self) -> Payload {
        match serde_json::from_str(&self.invoice_payload) {
            Ok(payload) => payload,
            Err(e) => panic!("Could not parse pre_checkout_query.payload. Error: {}", e),
        }
    }
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

// Send-Types
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

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceReplyMessage {
    // Must be sendInvoice
    pub method: String,
    pub chat_id: i32,
    // 1-32 Chars
    pub title: String,
    // 1-255 Chars
    pub description: String,
    // 1-128 Chars (not showed to user)
    pub payload: String,
    pub provider_token: String,
    pub start_parameter: String,
    pub currency: String,
    pub prices: Vec<LabeledPrice>,
    pub provider_data: Option<String>,
    pub photo_url: Option<String>,
    // pub photo_width: i32,
    // pub photo_height: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LabeledPrice {
    label: String,
    // In cents (1.50EUR = 150)
    amount: i32,
}
impl LabeledPrice {
    pub fn new(label: &str, amount: i32) -> Self {
        LabeledPrice {
            label: label.to_string(),
            amount,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreCheckoutQueryResponseMessage {
    // answerPreCheckoutQuery
    pub method: String,
    pub pre_checkout_query_id: String,
    pub ok: bool,
    pub error_message: Option<String>,
}
impl PreCheckoutQueryResponseMessage {
    pub fn new(id: &str, is_checkout_granted: bool) -> PreCheckoutQueryResponseMessage {
        PreCheckoutQueryResponseMessage {
            method: "answerPreCheckoutQuery".to_string(),
            pre_checkout_query_id: id.to_string(),
            ok: is_checkout_granted,
            // While there are no options and a donation can not be out of stock,
            // we only send an error if the total is above a certain threshold (for security reasons).
            error_message: if is_checkout_granted {
                None
            } else {
                Some("The total was to high. Transaction denied for security purposes.".to_string())
            },
        }
    }
}
