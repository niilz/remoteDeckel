// To choose DB first set ROCKET_ENV (in fish: set -x ROCKET_ENV dev)
// DBs are configured in rocekt.toml for dev-environment
//
// To use http tunnel pass use flag "url" and the tunnel url (cargo run url https://tunnelurl.com)
//
// To swap bot and use remoteDeckelTest_bot pass "test" flag
// - (cargo run test)
// - or with url (cargo run url https://tunnerlurl.com test)
//
#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel_migrations;

use bot_lib::bot_context::BotContext;
use bot_lib::bot_types::Keyboards;
use bot_lib::telegram_types::{self, Update};
use bot_lib::{db, models};
use dotenv::dotenv;
use reqwest;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::response::content;
use rocket::{post, routes, Outcome, Rocket};
use rocket_contrib::json::Json;

embed_migrations!();

static HOUR: i64 = 3600;

struct MonitoringRequest(String);

impl<'a, 'r> FromRequest<'a, 'r> for MonitoringRequest {
    type Error = String;
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        // Checks for the clever cloud monitoring header. It's value is "telegraf"
        let mut maybe_header = request.headers().get("X-Clevercloud-Monitoring");
        match maybe_header.next() {
            Some(value) => Outcome::Success(MonitoringRequest(value.to_string())),
            None => Outcome::Failure((
                Status::BadRequest,
                "Only Monitoring-GET-requests are supported".to_string(),
            )),
        }
    }
}

#[get("/")]
fn handle_get(_metrics_request: MonitoringRequest) -> String {
    // Only accepts a request if the clever-cloud-monitoring-header is present
    // It does nothing with the request but accepting it prevents the log from being cluttered
    "Received Monitoring GET-Request from clever cloud".to_string()
}

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
    let timestamp = incoming_message.date as i64 + (HOUR * 2);
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
    let user_name = match telegram_user.username {
        Some(ref username) => username,
        None => "undefined",
    };
    let first_name = telegram_user.first_name.to_string();
    let last_name = match telegram_user.last_name {
        Some(ref last_name) => last_name,
        None => "undefined",
    };
    let new_user = models::NewUser {
        id: telegram_user.id,
        name: &user_name,
        first_name: &first_name,
        last_name: last_name,
    };
    db::save_user(new_user, conn)
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

fn get_args() -> Vec<String> {
    std::env::args().collect()
}

fn get_ngrok_url() -> Option<String> {
    let args = get_args();
    match args.len() {
        3 | 4 if args[1] == "url" => Some(args[2].to_string()),
        _ => None,
    }
}

fn is_test() -> bool {
    let args = get_args();
    match args.len() {
        4 if args[3] == "test" => true,
        2 if args[1] == "test" => true,
        _ => false,
    }
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
        .mount("/", routes![handle_update, handle_get])
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
            eprintln!("Failed to run DB migration: {:?}", e);
            Err(rocket)
        }
    }
}

#[tokio::main]
async fn main() -> reqwest::Result<()> {
    // Set env-variables (port and postgres-db)
    dotenv().ok();

    let api_key = match is_test() {
        true => std::env::var("API_KEY_TEST"),
        false => std::env::var("API_KEY"),
    };
    let hosting_url = match get_ngrok_url() {
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
