#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
extern crate rocket;

pub mod bot_context;
pub mod bot_types;
pub mod db;
pub mod messages;
pub mod models;
pub mod payments;
pub mod schema;
pub mod stripe_types;
pub mod telegram_types;

pub fn get_args() -> Vec<String> {
    std::env::args().collect()
}

pub fn get_ngrok_url() -> Option<String> {
    let args = get_args();
    match args.len() {
        3 | 4 if args[1] == "url" => Some(args[2].to_string()),
        _ => None,
    }
}

pub fn is_test() -> bool {
    let args = get_args();
    match args.len() {
        4 if args[3] == "test" => true,
        2 if args[1] == "test" => true,
        _ => false,
    }
}
