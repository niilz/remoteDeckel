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

#[derive(Debug, AsChangeset, Default)]
#[table_name = "users"]
pub struct UpdateUser {
    pub id: i32,
    pub drink_count: Option<i16>,
    pub price: Option<PgMoney>,
    pub last_paid: Option<PgTimestamp>,
    pub last_total: Option<PgMoney>,
    pub total: Option<PgMoney>,
}

impl UpdateUser {
    pub fn from_user(user: &User) -> UpdateUser {
        UpdateUser {
            id: user.id,
            drink_count: Some(user.drink_count),
            price: Some(user.price),
            last_paid: Some(user.last_paid),
            last_total: Some(user.last_total),
            total: Some(user.last_total),
        }
    }
    // fn with_drink_count(count: i16) -> UpdateUser {
    //     let mut update_user = UpdateUser::default();
    //     update_user.drink_count = Some(count + count);
    //     update_user
    // }
    // fn erase_drinks() -> UpdateUser {
    //     let mut update_user = UpdateUser::default();
    //     update_user.drink_count = Some(0);
    //     update_user
    // }
}
