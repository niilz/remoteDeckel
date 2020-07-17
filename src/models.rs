use crate::schema::{payments, users};
use diesel::data_types::{PgMoney, PgTimestamp};
use diesel::{Identifiable, Insertable, Queryable};
// Order must be the same as the columns (http://diesel.rs/guides/getting-started/)
#[derive(Debug, Queryable, Identifiable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub first_name: String,
    pub last_name: String,
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
    pub first_name: &'a str,
    pub last_name: &'a str,
}

#[derive(Debug, AsChangeset, Default)]
#[table_name = "users"]
pub struct UpdateUser {
    pub drink_count: Option<i16>,
    pub price: Option<PgMoney>,
    pub last_paid: Option<PgTimestamp>,
    pub last_total: Option<PgMoney>,
    pub total: Option<PgMoney>,
}

impl UpdateUser {
    pub fn from_user(user: &User) -> UpdateUser {
        UpdateUser {
            drink_count: Some(user.drink_count),
            price: Some(user.price),
            last_paid: Some(user.last_paid),
            last_total: Some(user.last_total),
            total: Some(user.last_total),
        }
    }
}

#[derive(Debug, Queryable, Identifiable)]
pub struct Payment {
    pub id: i32,
    pub user_id: i32,
    pub receipt_identifier: String,
    pub payed_amount: PgMoney,
    pub payed_at: PgTimestamp,
    pub transfer_id: Option<String>,
}

#[derive(Debug, Insertable)]
#[table_name = "payments"]
pub struct NewPayment<'a> {
    pub user_id: i32,
    pub receipt_identifier: &'a str,
    pub payed_amount: PgMoney,
    pub payed_at: PgTimestamp,
}
