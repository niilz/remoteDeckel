
    // Test DB Connection
    use deckel_bot::schema::users;
    use deckel_bot::schema::users::dsl::*;
    use models::{NewUser, User};

    let new_user = NewUser { id: 4, name: "bob" };

    let connection = deckel_bot::establish_connection();

    let new_user: User = diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(&connection)
        .expect("Error saving user.");
    println!("inserted user: {:?}", new_user);

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
    let num = diesel::delete(users.filter(name.like("%hans%")))
        .execute(&connection)
        .expect("Could not delete Hanses");
    println!("DELETED: {}", num);
