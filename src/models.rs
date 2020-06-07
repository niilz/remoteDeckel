use diesel::data_types::{PgMoney, PgTimestamp};
use diesel::Queryable;
// Order must be the same as the columns (http://diesel.rs/guides/getting-started/)
#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub drink_count: i32,
    pub price: PgMoney,
    pub last_paid: PgTimestamp,
    pub last_total: PgMoney,
}
