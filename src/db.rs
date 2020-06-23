use crate::models;
use crate::schema::users::dsl::{drink_count, name, total, users};
use crate::telegram_types;
use diesel::data_types::PgMoney;
use diesel::prelude::*;
use rocket_contrib::databases::diesel::PgConnection;

#[database("remote_deckel")]
pub struct UserDbConn(PgConnection);

pub fn save_user(new_user: models::NewUser, conn: &PgConnection) -> models::User {
    let new_user = diesel::insert_into(users)
        .values(new_user)
        .get_result(conn)
        .expect("Error saving user.");
    new_user
}

pub fn get_user_by_id(
    given_id: i32,
    conn: &PgConnection,
) -> Result<models::User, diesel::result::Error> {
    users.find(given_id).first(conn)
}

// pub fn increase_drink_count_by_amount(
//     amount: i16,
//     user: models::User,
//     conn: &PgConnection,
// ) -> usize {
//     let updated_count = diesel::update(user)
//         .set(drink_count.eq(drink_count + amount))
//         .execute(conn)
//         .expect("Could not increase order");
//     updated_count
// }

pub fn update_user(user: models::UpdateUser, conn: &PgConnection) -> usize {
    let updated_count = diesel::update(users)
        .set(user)
        .execute(conn)
        .expect("Could not update user by given UpdateUser");
    updated_count
}

pub fn delete_user(user: &models::User, conn: &PgConnection) -> usize {
    let deleted_count = diesel::delete(user)
        .execute(conn)
        .expect("Could not delete given user");
    deleted_count
}

pub fn get_total_all(conn: &PgConnection) -> Vec<PgMoney> {
    users
        .select(total)
        .load::<PgMoney>(conn)
        .expect("Could not sum the total of all users")
}

pub fn show(conn: &PgConnection) {
    let results = users
        .filter(name.eq("hans"))
        .limit(10)
        .load::<models::User>(conn)
        .expect("Error loading Users");
    println!("Start Result-Printing");
    for user in results {
        println!("{:?}", user.last_paid);
        println!("{:?}", user.last_total);
    }
    println!("Stop Result-Printing");
}
