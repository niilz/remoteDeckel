// Version 0.1

#![feature(proc_macro_hygiene, decl_macro)]

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

#[post("/", format = "json", data = "<update>")]
fn take_order(update: Option<Json<Update>>) -> content::Json<String> {
    let response_message = match update {
        Some(data) => {
            println!("Incoming-Update: {:?}", data);
            construct_response(data)
        }
        None => panic!("Could not parse incoming Update-json"),
    };

    let response_as_json = match response_message {
        Ok(json) => json,
        Err(e) => panic!("{}", e),
    };
    content::Json(response_as_json)
}

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

fn construct_response(json_data: Json<Update>) -> serde_json::Result<String> {
    let method = "sendMessage".to_string();
    let update_message = match &json_data.message {
        Some(message) => message,
        None => panic!("No message in Update-Json"),
    };
    let request_text = match &update_message.text {
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
    let chat_id = update_message.chat.id;
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
    let bot_base_url = "https://3ec64f547f45.ngrok.io";
    let telegram_set_webhook_url = format!(
        "{}?url={}",
        bot_method_url("setWebhook", api_key),
        bot_base_url
    );
    println!(
        "Tries to register webHook with GET to: {}",
        telegram_set_webhook_url
    );

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

    // Test DB Connection
    use deckel_bot::schema::users;
    use deckel_bot::schema::users::dsl::*;
    use models::{NewUser, User};

    let new_user = NewUser {
        id: 1,
        name: "hans",
    };

    let connection = deckel_bot::establish_connection();

    let new_user: User = diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(&connection)
        .expect("Error saving user.");
    println!("inserted user: {:?}", new_user);

    let results = users
        .filter(name.eq("hans"))
        .limit(2)
        .load::<models::User>(&connection)
        .expect("Error loading Users");
    println!("Start Result-Printing");
    for user in results {
        println!("{:?}", user.last_paid);
        println!("{:?}", user.last_total);
    }
    println!("Stop Result-Printing");
    // Setup routes
    rocket::ignite().mount("/", routes![take_order]).launch();
    Ok(())
}
