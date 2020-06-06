// Version 0.1
// How to in Development:
// - set -x ROCKET_PORT 8443 (telgram only allows certain ports)
// - ngrok http 8443 (launches ngrok to re-route traffic through a "real" ip-address)
// - set ngrok's https url as "telegram_base_url"

#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use deckel_bot::*;
use reqwest::{self, Client};
use rocket::response::{content, status};
use rocket::State;
use rocket_contrib::json::Json;
use serde_json;
use serde_yaml;
use std::collections::BTreeMap;
use tokio::main;

#[get("/")]
fn index() -> &'static str {
    "Hello World"
}

#[post("/", format = "json", data = "<update>")]
fn take_order(update: Option<Json<Update>>) -> content::Json<String> {
    let message = match update {
        Some(data) => {
            println!("{:?}", data);
            process_order(data)
        }
        None => panic!("could not parse json"),
    };
    let res = ResponseMessage {
        method: "sendMessage".to_string(),
        chat_id: 123456,
        text: "It Works!".to_string(),
    };

    println!("RES: {:?}", res);
    let json = match serde_json::to_string(&res) {
        Ok(json) => json,
        Err(e) => panic!("{}", e),
    };
    content::Json(json)
}
fn test_print() {
    println!("TEST");
}

fn process_order(json_data: Json<Update>) -> serde_json::Result<String> {
    let json_res = match &json_data.message {
        Some(message) => match &message.text {
            Some(text) if text.to_lowercase() == "bier" => SendMessage {
                chat_id: message.chat.id,
                text: "Vielen Dank für deine Bestellung.".to_string(),
            },
            Some(text) => SendMessage {
                chat_id: message.chat.id,
                text: format!(
                    "Tut mir leid, {} haben wir nicht. Hier gibt's nur Bier.",
                    text
                ),
            },
            None => SendMessage {
                chat_id: message.chat.id,
                text: "oops, da ging gehörig was schief.".to_string(),
            },
        },
        None => {
            eprintln!("Could not parse incoming message.");
            unreachable!();
        }
    };
    serde_json::to_string(&json_res)
}

fn bot_method_url(method: &str, api_key: &str) -> String {
    let telegram_base_url = "https://api.telegram.org/bot";
    format!("{}{}/{}", telegram_base_url, api_key, method)
}

#[tokio::main]
async fn main() -> reqwest::Result<()> {
    // Get api_key from config-file
    let config_yml = std::fs::File::open("config.yml").expect("Could not read config.yml");
    let config_yml: BTreeMap<String, String> =
        serde_yaml::from_reader(config_yml).expect("Could not convert yml to serde_yaml");
    let api_key = config_yml.get("apikey").unwrap();

    // register update webHook
    let bot_base_url = "https://74832e788076.ngrok.io";
    let telegram_set_webhook_url = format!(
        "{}?url={}",
        bot_method_url("setWebhook", api_key),
        bot_base_url
    );
    println!("setWebhook command: {}", telegram_set_webhook_url);

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

    // Setup routes and state
    rocket::ignite().mount("/", routes![take_order]).launch();
    Ok(())
}
