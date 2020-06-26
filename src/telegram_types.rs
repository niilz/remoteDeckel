use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Update {
    pub update_id: i32,
    pub message: Option<Message>,
    pub pre_checkout_query: Option<PreCheckoutQuery>,
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
    pub provider_data: String,
    pub photo_url: String,
    // pub photo_width: i32,
    // pub photo_height: i32,
    // pub reply_markup: InlineKeyboardMarkup,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LabeledPrice {
    label: String,
    // In cents (1.50EUR = 150)
    amount: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Invoice {
    title: String,
    description: String,
    start_parameter: String,
    currency: String,
    // In cents (1.50EUR = 150)
    total_amount: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessfulPayment {
    currency: String,
    // In cents (1.50EUR = 150)
    total_amount: i32,
    invoice_payload: String,
    telegram_payment_charge_id: String,
    provider_payment_charge_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreCheckoutQuery {
    id: String,
    from: User,
    currency: String,
    // In cents (1.50EUR = 150)
    total_amount: i32,
    invoice_payload: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreCheckoutQueryResponseMessage {
    // answerPreCheckoutQuery
    method: String,
    pre_checkout_query_id: String,
    ok: bool,
    error_message: Option<String>,
}
