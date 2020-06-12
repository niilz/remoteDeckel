// Version 0.1

#![feature(proc_macro_hygiene, decl_macro)]

use decke_bot::db;
use deckel_bot;
use deckel_bot::models;
use deckel_bot::telegram_types::{ReplyKeyboardMarkup, ResponseMessage, Update};
use diesel::prelude::*;
use dotenv::dotenv;
use reqwest;
use rocket::response::content;
use rocket::{post, routes};
use rocket_contrib::json::Json;
use serde_json;
use serde_yaml;
use std::collections::BTreeMap;
use tokio;

static WELCOME_MESSAGE: &'static str = r"Willkommen Mensch!

Ich bin's, der remoteDeckel_bot.

Zusammen können wir saufen UND unsere Lieblingskneipe supporten.

Bestell' einfach deine Biers bei mir und ich schreib' sie auf deinen Deckel.

Du bestimmst wieviel du pro Getränk spenden möchtest.

Wenn OberkanteUnterlippe erreicht ist, gib mir Bescheid, um den 'Schaden' zu begleichen:
- Dann wird dein Deckel genullt
- und deine Spende übermittelt

Und keine Sorge. Wenn der Durst doch größer war als es die Haushaltskasse erlaubt. Du kannst jederzeit den Spendenbetrag reduzieren oder die ganze Zeche prellen.

Na dann, Prost!";

#[post("/", format = "json", data = "<update>")]
fn handle_update(dbConn: db::UserDbConn, update: Option<Json<Update>>) -> content::Json<String> {
    let response_message = match update {
        Some(update) => {
            println!("Incoming-Update: {:?}", update);
            react(update, dbConn)
        }
        None => panic!("Could not parse incoming Update-json"),
    };

    let response_as_json = match response_message {
        Ok(json) => json,
        Err(e) => panic!("{}", e),
    };
    content::Json(response_as_json)
}

fn react(update: Json<Update>, dbConn: db::UserDbConn) -> serde_json::Result<String> {
    let message = match update.message {
        Some(message) => message,
        None => panic!("update had not message"),
    };
    let user = match message.from {
        Some(user) => user,
        None => panic!("message has no sender?..."),
    };
    println!("USER: {:?}", user);
    // Does user exist?
    let db_user = models::NewUser::from_telegram_user(user);
    println!("DB_User: {:?}", db_user);
    // get user-data
    // otherwise put user in db

    let method = "sendMessage".to_string();
    let request_text = match &message.text {
        Some(text) => text.to_lowercase(),
        None => panic!("No text in Message!"),
    };
    let response_text = if request_text == "/start" {
        WELCOME_MESSAGE.to_string()
    } else if request_text.contains("bier") {
        // TODO: "Increase tab-count"
        "👍 Ich schreib's auf deinen Deckel.".to_string()
    } else if request_text.contains("schaden") {
        // TODO: persist actual damage
        let damage = 42;
        format!("Dein derzeitiger Deckel beträgt {},-€.", damage)
    } else if request_text.contains("zahlen") {
        // TODO: initiate payment
        let total = 199;
        format!(
            "🙏 Danke für deine Spende 🙏\n💶 in Höhe von {},-€ 💶\n🦸 Du bist ein Retter! 🦸",
            total
        )
    } else {
        "Ehm, darauf weiß ich keine Antwort...".to_string()
    };
    let chat_id = message.chat.id;
    let response_message = ResponseMessage::new(method, chat_id, response_text);
    let response_message = response_message.keyboard(ReplyKeyboardMarkup::default());
    serde_json::to_string(&response_message)
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
    let config_yml = std::fs::File::open("config.yml").expect("Could not read config.yml");
    let config_yml: BTreeMap<String, String> =
        serde_yaml::from_reader(config_yml).expect("Could not convert yml to serde_yaml");
    let api_key = config_yml.get("apikey").unwrap();

    // Register update webHook with Telegram
    // TODO: Automate ngrok setup, or actually host it
    let bot_base_url = "https://09612a2395eb.ngrok.io";
    let telegram_set_webhook_url = format!(
        "{}?url={}",
        bot_method_url("setWebhook", api_key),
        bot_base_url
    );
    println!(
        "Tries to register webHook with GET to: {}",
        telegram_set_webhook_url
    );
    eprintln!("Webhook setup disabled");
    // let webhook_response = reqwest::get(&telegram_set_webhook_url)
    //     .await?
    //     .text()
    //     .await?;
    // println!("SetWebhook-Response: {:?}", webhook_response);

    // let webhook_info = reqwest::get(&bot_method_url("getWebhookInfo", api_key))
    //     .await?
    //     .text()
    //     .await?;
    // println!("Webhook-Info: {:?}", webhook_info);

    // Setup routes
    rocket::ignite()
        .mount("/", routes![handle_update])
        .attach(db::UserDbConn::fairing())
        .launch();
    Ok(())
}
