use crate::models::{NewUser, User};
use crate::schema::users::dsl::{drink_count, id, name, users};
use crate::telegram_types;
use diesel::prelude::*;
use rocket_contrib::databases::diesel::PgConnection;

#[database("remote_deckel")]
pub struct UserDbConn(PgConnection);

// pub fn establish_connection() -> PgConnection {
//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
// }
// let connection = deckel_bot::establish_connection();
pub fn save_user(new_user: &telegram_types::User, conn: &PgConnection) -> User {
    let user_name = match new_user.username {
        Some(ref username) => username,
        None => "undefined",
    };
    let new_user = NewUser {
        id: new_user.id,
        name: &user_name,
    };
    let new_user = diesel::insert_into(users)
        .values(&new_user)
        .get_result(conn)
        .expect("Error saving user.");
    new_user
}

pub fn get_user_by_id(given_id: i32, conn: &PgConnection) -> Result<User, diesel::result::Error> {
    users.find(given_id).first(conn)
}

pub fn increase_order(amount: i16, conn: &PgConnection) -> usize {
    let updated_count = diesel::update(users)
        .set(drink_count.eq(drink_count + amount))
        .execute(conn)
        .expect("Could not increase order");
    updated_count
}

pub fn delete_users_by_name(user_name: &str, conn: &PgConnection) -> usize {
    let deleted_count = diesel::delete(users.filter(name.eq(user_name)))
        .execute(conn)
        .expect("Could not delete given user");
    deleted_count
}
pub fn delete_user_by_id(given_id: i32, conn: &PgConnection) -> Result<i32, diesel::result::Error> {
    let deleted_id = diesel::delete(users.filter(id.eq(&given_id)))
        .returning(id)
        .get_result(conn);
    deleted_id
}

pub fn show(conn: &PgConnection) {
    let results = users
        .filter(name.eq("hans"))
        .limit(10)
        .load::<User>(conn)
        .expect("Error loading Users");
    println!("Start Result-Printing");
    for user in results {
        println!("{:?}", user.last_paid);
        println!("{:?}", user.last_total);
    }
    println!("Stop Result-Printing");
}
