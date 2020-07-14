use crate::db;
use crate::models::{NewPayment, UpdateUser};
use crate::stripe_types::*;
use crate::telegram_types::SuccessfulPayment;
use chrono::{DateTime, Duration, TimeZone, Utc};
use diesel::pg::data_types::PgTimestamp;
use diesel::pg::types::money::PgMoney;
use reqwest::blocking::Client;
use reqwest::header::{self, HeaderMap};
use serde_json;

pub fn pay(
    successful_payment: &SuccessfulPayment,
    conn: db::UserDbConn,
) -> Result<(), reqwest::Error> {
    persist_payment(successful_payment, &conn);

    let stripe_token = std::env::var("STRIPE_TOKEN_TEST").unwrap();
    let client = Client::builder().build()?;
    let balance = get_balance(&client, &stripe_token);
    println!("Current Balance: {:?}", balance);
    match balance {
        Ok(balance) if balance < successful_payment.total_amount => {
            println!("Balance does not cover amount")
        }
        Ok(balance) => println!("balance is: {:?}", balance),
        Err(e) => eprintln!("Could not get balance. Err: {}", e),
    }
    let payment_intent =
        payment_intent_request(&client, &stripe_token, successful_payment.total_amount);
    let confirm_request = match payment_intent {
        Ok(pi) => confirm_payment(&pi.id, &client, &stripe_token),
        Err(e) => Err(e),
    };
    println!("confirm_request un-de: {:?}", confirm_request);
    match confirm_request {
        Ok(confirm) => println!(
            "Great a transfer of {} has been sent to the pub",
            confirm.amount
        ),
        Err(e) => eprintln!("Sad but conirmation failed"),
    }
    Ok(())
}

fn persist_payment(successful_payment: &SuccessfulPayment, conn: &db::UserDbConn) {
    let payload = successful_payment.get_payload();

    let last_paid = (Utc::now() + Duration::hours(2)).timestamp();
    let new_last_total = payload.total;
    let total = payload.totals_sum + new_last_total;
    let mut update_user = UpdateUser::default();

    update_user.last_paid = Some(PgTimestamp(last_paid));
    update_user.last_total = Some(PgMoney(new_last_total));
    update_user.total = Some(PgMoney(total));
    update_user.drink_count = Some(0);
    db::update_user(payload.user_id, &update_user, &conn);

    let new_payment = NewPayment {
        user_id: payload.user_id,
        receipt_identifier: &successful_payment.provider_payment_charge_id,
        payed_amount: PgMoney(payload.total),
        payed_at: PgTimestamp(last_paid),
    };
    db::save_payment(new_payment, &conn);
}

fn payment_intent_request(
    client: &Client,
    token: &str,
    amount: i32,
) -> Result<PaymentIntent, reqwest::Error> {
    let destination_account = std::env::var("DESTINATION").unwrap();
    let payment_intent = &[
        ("payment_method_types[]", "card"),
        ("amount", &amount.to_string()),
        ("currency", "eur"),
        ("transfer_data[destination]", &destination_account),
    ];

    let res = client
        .post("https://api.stripe.com/v1/payment_intents")
        .bearer_auth(&token)
        .form(payment_intent)
        .send();

    match res {
        Ok(payment_intent_res) => match payment_intent_res.json::<PaymentIntent>() {
            Ok(pi) => {
                println!("PaymentIntent responded with: {:?}", pi);
                Ok(pi)
            }
            Err(e) => {
                eprintln!("Could not Deserialize payment_intent. Err: {}", e);
                Err(e)
            }
        },
        Err(e) => {
            eprintln!("Could not reslove payment_intent_request. Err: {}", e);
            Err(e)
        }
    }
}

fn confirm_payment(
    payment_id: &str,
    client: &Client,
    token: &str,
) -> Result<PaymentConfirmation, reqwest::Error> {
    let url = format!(
        "https://api.stripe.com/v1/payment_intents/{}/confirm",
        payment_id
    );
    let res = client
        .post(&url)
        .bearer_auth(&token)
        // TODO: Use actual card-information
        .form(&[("payment_method", "pm_card_visa")])
        .send()?;
    res.json::<PaymentConfirmation>()
}

pub fn get_balance(client: &Client, token: &str) -> Result<i32, reqwest::Error> {
    let res = client
        .get("https://api.stripe.com/v1/balance")
        .bearer_auth(token)
        .send();
    let balance: Balance = res?.json()?;
    Ok(balance.pending.first().unwrap().amount)
}

// Helpers
pub fn money_in_eur(money: i64) -> f32 {
    money as f32 / 100.00
}
