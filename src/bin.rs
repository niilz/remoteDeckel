// Version 0.1
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel_migrations;

use bot_lib::bot_types::{Keyboards, RequestType};
use bot_lib::telegram_types::{self, ReplyKeyboardMarkup, ResponseMessage, Update};
use bot_lib::{bot_types::keyboard_factory, db, messages, models};
use chrono::NaiveDateTime;
use diesel::data_types::{PgMoney, PgTimestamp};
use dotenv::dotenv;
use reqwest;
use rocket::fairing::AdHoc;
use rocket::response::content;
use rocket::{post, routes, Rocket};
use rocket_contrib::json::Json;
use serde_json;
use serde_yaml;
use std::collections::BTreeMap;
use tokio;

embed_migrations!();

#[post("/", format = "json", data = "<update>")]
fn handle_update(conn: db::UserDbConn, update: Json<Update>) -> content::Json<String> {
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
        Err(_) => {
            let new_user = persist_new_user(&telegram_user, &conn);
            println!(
                "New user: {} with id: {} has been created",
                new_user.name, new_user.id
            );
            new_user
        }
    };
    let chat_id = incoming_message.chat.id;
    let user_text = get_text_from_message(&incoming_message);
    let timestamp = incoming_message.date;
    let keyboards = Keyboards::init();
    let mut bot_context =
        BotContext::new(current_user, conn, chat_id, user_text, timestamp, keyboards);
    let request_type = bot_context.get_request_type(&incoming_message);

    let json_response = match bot_context.handle_request(request_type) {
        Ok(json) => json,
        Err(e) => panic!("{}", e),
    };
    content::Json(json_response)
}

fn get_user_from_db(
    telegram_user: &telegram_types::User,
    conn: &db::UserDbConn,
) -> Result<models::User, diesel::result::Error> {
    db::get_user_by_id(telegram_user.id, conn)
}

fn persist_new_user(telegram_user: &telegram_types::User, conn: &db::UserDbConn) -> models::User {
    // Delete old webHook if it exists
    db::save_user(telegram_user, conn)
}

struct BotContext {
    current_user: models::User,
    conn: db::UserDbConn,
    chat_id: i32,
    request_message: String,
    timestamp: NaiveDateTime,
    keyboards: Keyboards,
}

impl BotContext {
    fn new(
        current_user: models::User,
        conn: db::UserDbConn,
        chat_id: i32,
        request_message: String,
        timestamp: i32,
        keyboards: Keyboards,
    ) -> Self {
        BotContext {
            current_user,
            conn,
            chat_id,
            request_message: request_message.to_string(),
            timestamp: NaiveDateTime::from_timestamp(timestamp as i64, 0),
            keyboards,
        }
    }

    fn handle_request(&mut self, request_type: RequestType) -> serde_json::Result<String> {
        let response_text = match request_type {
            RequestType::Start => messages::WELCOME_MESSAGE.to_string(),
            RequestType::Order => {
                self.order_drink();
                "👍 Ich schreib's auf deinen Deckel.".to_string()
            }
            RequestType::ShowDamage => format!(
                "Du hast bisher {} Biers bestellt.\nBeim aktuellen Preis von {:.2}€ beträgt dein derzeitiger Deckel insgesamt {:.2}€.",
                self.current_user.drink_count,
                self.money_in_eur(self.current_user.price.0),
                self.money_in_eur(self.get_damage())
            ),
            RequestType::BillPlease => format!(
                "💶 Dein derzeitiger Schaden beträgt {:.2}€. 💶\nMöchtest du wirklich zahlen?",
                self.money_in_eur(self.get_damage())
            ),
            RequestType::PayNo => "Ok, dann lass uns lieber weiter trinken.".to_string(),
            RequestType::PayYes => {
                self.pay();
                format!(
                "🙏 Danke für deine Spende 🙏\n💶 in Höhe von {},-€ 💶\n🦸 Du bist ein Retter! 🦸",
                self.money_in_eur(self.current_user.last_total.0))
            }
            RequestType::DeletePlease => {
                "Möchtest du deine Userdaten wirklich löschen?".to_string()
            }
            RequestType::DeleteNo => "Ok, deine Daten wurden nicht gelöscht.".to_string(),
            RequestType::DeleteYes => {
                let deleted_user = self.delete_user();
                // TODO: delete returns count not deleted_user
                println!("User: {} with id: {} has been deleted", deleted_user, deleted_user);
                "No problemo. Ich habe deine Daten gelöscht.".to_string()
            }
            RequestType::Steal => {
                self.erase_drinks();
                "Ich habe deinen Deckel unauffällig zerrissen.".to_string()
            }
            RequestType::Options => "Was kann ich für dich tun?".to_string(),
            RequestType::ChangePrice => "Wähle einen neuen Getränkepreis.".to_string(),
            RequestType::NewPrice => {
                let new_price = self.convert_price();
                match new_price {
                    Ok(price) if price <= 200 => {
                        self.update_price(price);
                        format!(
                            "Alles klar, jedes Getränk kostet jetzt {:.2}€",
                            self.money_in_eur(self.current_user.price.0)
                        )
                    }
                    _ => "🥁 Sorry aber das ist hier kein Wunschkonzert 🥁".to_string(),
                }
            }
            RequestType::ShowLast => format!(
                "Deine letzte Spende war am {:?} und betrug {:.2}€.",
                self.get_last_paid_as_date(),
                self.money_in_eur(self.current_user.last_total.0)
            ),
            RequestType::ShowTotal => format!(
                "Insgesamt hast du {:.2}€ gespendet.",
                self.money_in_eur(self.current_user.total.0)
            ),
            RequestType::ShowTotalAll => format!(
                "Zusammen haben wir {:.2}€ gespendet.",
                db::get_total_all(&self.conn)
            ),
            RequestType::Unknown => {
                "🤷 Ehm, sorry darauf weiß ich grade keine Antwort...".to_string()
            }
        };
        let method = "sendMessage".to_string();
        let response_message = ResponseMessage::new(method, self.chat_id, response_text);
        let keyboard = self.provide_keyboard(request_type);
        let response_message = response_message.keyboard(keyboard);
        serde_json::to_string(&response_message)
    }

    fn order_drink(&mut self) {
        self.current_user.drink_count += 1;
        db::update_user(&self.current_user, &self.conn);
    }

    fn get_damage(&self) -> i64 {
        let drinks = self.current_user.drink_count;
        let price = self.current_user.price.0;
        drinks as i64 * price
    }

    fn money_in_eur(&self, money: i64) -> f32 {
        money as f32 / 100.00
    }

    fn convert_price(&self) -> Result<i64, std::num::ParseIntError> {
        let new_price = self.request_message.replace("€", "").replace(",", "");
        let new_price = if &new_price[..1] == "0" {
            new_price[1..].to_string()
        } else {
            new_price
        };
        new_price.parse::<i64>()
    }

    fn update_price(&mut self, new_price: i64) {
        self.current_user.price = PgMoney(new_price);
        db::update_user(&self.current_user, &self.conn);
    }

    fn pay(&mut self) {
        self.current_user.last_paid = PgTimestamp(self.timestamp.timestamp());
        let new_last_total = PgMoney(self.get_damage());
        self.current_user.last_total = new_last_total;
        self.current_user.total = self.current_user.total + new_last_total;
        self.current_user.drink_count = 0;
        db::update_user(&self.current_user, &self.conn);
    }

    fn erase_drinks(&mut self) {
        self.current_user.drink_count = 0;
        self.current_user.total = PgMoney(0);
        db::update_user(&self.current_user, &self.conn);
    }

    fn get_last_paid_as_date(&self) -> String {
        let date_time = NaiveDateTime::from_timestamp(self.current_user.last_paid.0, 0);
        date_time.format("%d.%m.%Y um %H:%Mh").to_string()
    }

    fn delete_user(&self) -> usize {
        db::delete_user(&self.current_user, &self.conn)
    }

    fn get_request_type(&self, message: &telegram_types::Message) -> RequestType {
        let request_message = match &message.text {
            Some(text) => text.to_string(),
            None => match &message.sticker {
                Some(sticker) => match &sticker.emoji {
                    Some(emoji) => emoji.to_string(),
                    None => return RequestType::Unknown,
                },
                None => return RequestType::Unknown,
            },
        };
        if request_message == "/start" {
            return RequestType::Start;
        }
        self.keyboards.get_request_type(&request_message)
    }

    fn provide_keyboard(&self, request_type: RequestType) -> ReplyKeyboardMarkup {
        match request_type {
            RequestType::BillPlease => keyboard_factory(&self.keyboards.pay),
            RequestType::DeletePlease => keyboard_factory(&self.keyboards.delete),
            RequestType::Options => keyboard_factory(&self.keyboards.options),
            RequestType::ChangePrice => keyboard_factory(&self.keyboards.price),
            _ => keyboard_factory(&self.keyboards.main),
        }
    }
}

fn get_text_from_message(telegram_message: &telegram_types::Message) -> String {
    match &telegram_message.text {
        Some(text) => text.to_string(),
        None => "".to_string(),
    }
}

fn bot_method_url(method: &str, api_key: &str) -> String {
    let telegram_base_url = "https://api.telegram.org/bot";
    format!("{}{}/{}", telegram_base_url, api_key, method)
}

fn get_ngrok_url_arg() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        3 if args[1] == "u" => Some(args[2].to_string()),
        _ => None,
    }
}

fn get_config() -> BTreeMap<String, String> {
    let config_yml = std::fs::File::open("config.yml").expect("Could not read config.yml");
    serde_yaml::from_reader(config_yml).expect("Could not convert yml to serde_yaml")
}

async fn set_webhook(bot_base_url: &str, api_key: &str) -> reqwest::Result<()> {
    // Register update webHook with Telegram
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
    Ok(())
}

fn launch_rocket() {
    rocket::ignite()
        .mount("/", routes![handle_update])
        .attach(db::UserDbConn::fairing())
        .attach(AdHoc::on_attach("Database Migration", run_db_migrations))
        .launch();
}

// see: https://stackoverflow.com/questions/61047355/how-to-run-diesel-migration-with-rocket-in-production
// and: https://docs.rs/crate/diesel_migrations/1.4.0
fn run_db_migrations(rocket: Rocket) -> Result<Rocket, Rocket> {
    let conn =
        db::UserDbConn::get_one(&rocket).expect("Could not establish rocket with DB connection");
    match embedded_migrations::run(&*conn) {
        Ok(()) => Ok(rocket),
        Err(e) => {
            eprintln!("Failed to run db migration: {:?}", e);
            Err(rocket)
        }
    }
}

#[tokio::main]
async fn main() -> reqwest::Result<()> {
    // Set env-variables (port and postgres-db)
    dotenv().ok();

    let api_key = std::env::var("API_KEY");
    let hosting_url = match get_ngrok_url_arg() {
        Some(url) => Ok(url),
        None => std::env::var("HOSTING_URL"),
    };
    match (&hosting_url, &api_key) {
        (Ok(url), Ok(key)) => set_webhook(url, key).await?,
        _ => eprintln!("Webhook setup disabled"),
    }

    launch_rocket();
    Ok(())
}
