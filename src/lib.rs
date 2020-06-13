#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket;

pub mod db;
pub mod models;
pub mod schema;
pub mod telegram_types;
