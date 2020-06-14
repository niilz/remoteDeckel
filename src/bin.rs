// Version 0.1
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use bot_lib::telegram_types::{self, ReplyKeyboardMarkup, ResponseMessage, Update};
use bot_lib::{db, messages, models};
use dotenv::dotenv;
use reqwest;
use rocket::response::content;
use rocket::{post, routes};
use rocket_contrib::json::Json;
use serde_json;
use serde_yaml;
use std::collections::BTreeMap;
use tokio;

struct BotContext {
    current_user: models::User,
    conn: db::UserDbConn,
    chat_id: i32,
}
impl BotContext {
    fn new(current_user: models::User, conn: db::UserDbConn, chat_id: i32) -> Self {
        BotContext {
            current_user,
            conn,
            chat_id,
        }
    }

    fn handle_request(&self, request_type: RequestType) -> serde_json::Result<String> {
        let response_text = match request_type {
            RequestType::Start => messages::WELCOME_MESSAGE.to_string(),
            RequestType::Order => {
                self.order_drink();
                "👍 Ich schreib's auf deinen Deckel.".to_string()
            }
            RequestType::ShowDamage => {
                format!("Dein derzeitiger Deckel beträgt {},-€.", self.get_damage())
            }
            RequestType::Pay => format!(
                "🙏 Danke für deine Spende 🙏\n💶 in Höhe von {},-€ 💶\n🦸 Du bist ein Retter! 🦸",
                self.get_damage()
            ),
            RequestType::Unknown => "Ehm, sorry darauf weiß ich grade keine Antwort...".to_string(),
        };
        let method = "sendMessage".to_string();
        let response_message = ResponseMessage::new(method, self.chat_id, response_text);
        let response_message = response_message.keyboard(ReplyKeyboardMarkup::default());
        serde_json::to_string(&response_message)
    }

    fn order_drink(&self) {
        db::increase_order(1, &self.conn);
    }

    fn get_damage(&self) -> f32 {
        let drinks = self.current_user.drink_count;
        let price = self.current_user.price.0;
        (drinks as i64 * price) as f32 / 100.00
    }
}

enum RequestType {
    Start,
    Order,
    ShowDamage,
    Pay,
    Unknown,
}

#[post("/", format = "json", data = "<update>")]
fn handle_update(conn: db::UserDbConn, update: Json<Update>) -> content::Json<String> {
    println!("Incoming-Update: {:?}", update);
    let incoming_message = match &update.message {
        Some(message) => message,
        None => panic!("No message?...TODO: http 500 response"),
    };
    let telegram_user = match &incoming_message.from {
        Some(user) => user,
        None => panic!("message has no sender?...(from = None)"),
    };
    let current_user = match get_user_from_db(&telegram_user, &conn) {
        Ok(user) => user,
        Err(_) => persist_new_user(&telegram_user, &conn),
    };
    let chat_id = incoming_message.chat.id;
    let bot_context = BotContext::new(current_user, conn, chat_id);
    let request_type = get_request_type(&incoming_message);

    let json_response = match bot_context.handle_request(request_type) {
        Ok(json) => json,
        Err(e) => panic!("{}", e),
    };
    content::Json(json_response)
}

fn get_request_type(message: &telegram_types::Message) -> RequestType {
    let request_message = match &message.text {
        Some(text) => text.to_lowercase().to_string(),
        None => panic!("No text in Message!"),
    };
    if request_message == "/start" {
        return RequestType::Start;
    } else if request_message.contains("bier") {
        return RequestType::Order;
    } else if request_message.contains("schaden") {
        return RequestType::ShowDamage;
    } else if request_message.contains("zahlen") {
        return RequestType::Pay;
    }
    RequestType::Unknown
}

fn get_user_from_db(
    telegram_user: &telegram_types::User,
    conn: &db::UserDbConn,
) -> Result<models::User, diesel::result::Error> {
    println!(
        "Tries to get user: {}, with id: {} from db.",
        telegram_user.first_name, telegram_user.id
    );
    db::get_user_by_id(telegram_user.id, conn)
}

fn persist_new_user(telegram_user: &telegram_types::User, conn: &db::UserDbConn) -> models::User {
    println!(
        "Saves user: {}, with id: {} to db.",
        telegram_user.first_name, telegram_user.id
    );
    db::save_user(telegram_user, conn)
}

fn bot_method_url(method: &str, api_key: &str) -> String {
    let telegram_base_url = "https://api.telegram.org/bot";
    format!("{}{}/{}", telegram_base_url, api_key, method)
}

#[tokio::main]
async fn main() -> reqwest::Result<()> {
    // Set env-variables (port and postgres-db)
    dotenv().ok();
    // Get api_key from config-file
    let config_yml = std::fs::File::open("../config.yml").expect("Could not read config.yml");
    let config_yml: BTreeMap<String, String> =
        serde_yaml::from_reader(config_yml).expect("Could not convert yml to serde_yaml");
    let api_key = config_yml.get("apikey").unwrap();

    // Register update webHook with Telegram
    // TODO: Automate ngrok setup, or actually host it
    let bot_base_url = "https://5919eabb94f6.ngrok.io";
    let telegram_set_webhook_url = format!(
        "{}?url={}",
        bot_method_url("setWebhook", api_key),
        bot_base_url
    );
    println!(
        "Tries to register webHook with GET to: {}",
        telegram_set_webhook_url
    );
    // eprintln!("Webhook setup disabled");
    let webhook_response = reqwest::get(&telegram_set_webhook_url)
        .await?
        .text()
        .await?;
    println!("SetWebhook-Response: {:?}", webhook_response);

    let webhook_info = reqwest::get(&bot_method_url("getWebhookInfo", api_key))
        .await?
        .text()
        .await?;
    println!("Webhook-Info: {:?}", webhook_info);

    // Setup routes
    rocket::ignite()
        .mount("/", routes![handle_update])
        .attach(db::UserDbConn::fairing())
        .launch();
    Ok(())
}
