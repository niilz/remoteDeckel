use crate::models;
use crate::schema;
use crate::telegram_types;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use rocket_contrib::databases;
use std::env;

#[database("remote_deckel")]
pub struct UserDbConn(diesel::PgConnection);

// pub fn establish_connection() -> PgConnection {
//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
// }
// let connection = deckel_bot::establish_connection();
pub fn save_user(new_user: models::User, conn: diesel::Connection) -> telegram_types::User {
    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(&conn)
        .expect("Error saving user.")
}

pub fn delete_users_by_name(user_name: &str, conn: diesel::Connection) -> i32 {
    let deleted_count = diesel::delete(users::table.filter(name.eq(user_name)))
        .execute(&conn)
        .expect("Could not delete given user");
    deleted_count
}
pub fn delete_user_by_id(id: i32, conn: diesel::Connection) -> Option<i32> {
    diesel::delete(users::table.filter(id.eq(id)))
        .returning(id)
        .get_result(&conn)
}

pub fn get_user_by_id(id: i32, conn: diesel::Connection) -> Option<models::User> {
    users.find(id).first(&conn)
}

pub fn show() {
    let results = users
        .filter(name.eq("hans"))
        .limit(10)
        .load::<models::User>(&connection)
        .expect("Error loading Users");
    println!("Start Result-Printing");
    for user in results {
        println!("{:?}", user.last_paid);
        println!("{:?}", user.last_total);
    }
    println!("Stop Result-Printing");
}
