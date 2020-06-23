use crate::schema::users;
use diesel::data_types::{PgMoney, PgTimestamp};
use diesel::{Identifiable, Insertable, Queryable};
// Order must be the same as the columns (http://diesel.rs/guides/getting-started/)
#[derive(Debug, Queryable, Identifiable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub drink_count: i16,
    pub price: PgMoney,
    pub last_paid: PgTimestamp,
    pub last_total: PgMoney,
    pub total: PgMoney,
}

#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub id: i32,
    pub name: &'a str,
}

#[derive(Debug, AsChangeset)]
#[table_name = "users"]
pub struct UpdateUser {
    pub drink_count: i16,
    pub price: PgMoney,
    pub last_paid: PgTimestamp,
    pub last_total: PgMoney,
    pub total: PgMoney,
}

impl UpdateUser {
    pub fn from_user(user: &User) -> UpdateUser {
        UpdateUser {
            drink_count: user.drink_count,
            price: user.price,
            last_paid: user.last_paid,
            last_total: user.last_total,
            total: user.last_total,
        }
    }
}
