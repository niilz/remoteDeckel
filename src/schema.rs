table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        first_name -> Varchar,
        last_name -> Varchar,
        drink_count -> Int2,
        price -> Money,
        last_paid -> Timestamp,
        last_total -> Money,
        total -> Money,
    }
}
