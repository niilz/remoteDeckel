table! {
    payments (id) {
        id -> Int4,
        user_id -> Int4,
        receipt_identifier -> Varchar,
        payed_amount -> Money,
        payed_at -> Timestamp,
        transfer_id -> Nullable<Varchar>,
    }
}

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

allow_tables_to_appear_in_same_query!(
    payments,
    users,
);
