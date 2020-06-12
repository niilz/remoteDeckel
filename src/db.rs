use crate::models::{NewUser, User};
use crate::schema::users::dsl::{id, name, users};
use crate::telegram_types;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use rocket_contrib::databases::diesel;

#[database("remote_deckel")]
pub struct UserDbConn(diesel::PgConnection);

// pub fn establish_connection() -> PgConnection {
//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
// }
// let connection = deckel_bot::establish_connection();
pub fn save_user(new_user: &telegram_types::User, conn: UserDbConn) -> (User, UserDbConn) {
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
        .get_result(&*conn)
        .expect("Error saving user.");
    (new_user, conn)
}

pub fn delete_users_by_name(user_name: &str, conn: UserDbConn) -> (usize, UserDbConn) {
    let deleted_count = diesel::delete(users.filter(name.eq(user_name)))
        .execute(&*conn)
        .expect("Could not delete given user");
    (deleted_count, conn)
}
pub fn delete_user_by_id(
    given_id: i32,
    conn: UserDbConn,
) -> (Result<i32, diesel::result::Error>, UserDbConn) {
    let deleted_id = diesel::delete(users.filter(id.eq(&given_id)))
        .returning(id)
        .get_result(&*conn);
    (deleted_id, conn)
}

pub fn get_user_by_id(
    given_id: i32,
    conn: UserDbConn,
) -> (Result<User, diesel::result::Error>, UserDbConn) {
    let maybe_user = users.find(given_id).first(&*conn);
    (maybe_user, conn)
}

pub fn show(conn: UserDbConn) {
    let results = users
        .filter(name.eq("hans"))
        .limit(10)
        .load::<User>(&*conn)
        .expect("Error loading Users");
    println!("Start Result-Printing");
    for user in results {
        println!("{:?}", user.last_paid);
        println!("{:?}", user.last_total);
    }
    println!("Stop Result-Printing");
}
