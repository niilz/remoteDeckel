#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;

pub mod bot_types;
pub mod db;
pub mod messages;
pub mod models;
pub mod schema;
pub mod telegram_types;
