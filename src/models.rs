use crate::schema::users;
use crate::telegram_types;
use diesel::data_types::{PgMoney, PgTimestamp};
use diesel::{Insertable, Queryable};
// Order must be the same as the columns (http://diesel.rs/guides/getting-started/)
#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub drink_count: i16,
    pub price: PgMoney,
    pub last_paid: PgTimestamp,
    pub last_total: PgMoney,
}

#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub id: i32,
    pub name: String,
}

impl NewUser {
    pub fn from_telegram_user(user: telegram_types::User) -> Self {
        let user_name = user.username.unwrap_or("undefined".to_string());
        NewUser {
            id: user.id,
            name: user_name,
        }
    }
}
