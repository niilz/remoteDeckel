use crate::models;
use crate::schema::payments::dsl::payments;
use crate::schema::users::dsl::{id, total, users};
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

pub fn update_user(user_id: i32, update_user: &models::UpdateUser, conn: &PgConnection) -> usize {
    let updated_count = diesel::update(users)
        .filter(id.eq(user_id))
        .set(update_user)
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

// PAYMENTS
pub fn save_payment(new_payment: models::NewPayment, conn: &PgConnection) -> models::Payment {
    diesel::insert_into(payments)
        .values(new_payment)
        .get_result(conn)
        .expect("Could not save new payment")
}
