use crate::db;
use crate::is_test;
use crate::models::{NewPayment, Payment, UpdateUser};
use crate::stripe_types::*;
use crate::telegram_types::SuccessfulPayment;
use chrono::{Duration, Utc};
use diesel::pg::data_types::PgTimestamp;
use diesel::pg::types::money::PgMoney;
use reqwest::blocking::Client;

pub fn pay(
    successful_payment: &SuccessfulPayment,
    conn: db::UserDbConn,
) -> Result<(), reqwest::Error> {
    // User has successfuly payed, so this fact is saved
    let payment = persist_payment(successful_payment, &conn);

    let stripe_token_str = if is_test() {
        "STRIPE_TOKEN_TEST"
    } else {
        "STRIPE_TOKEN"
    };

    let stripe_token = std::env::var(stripe_token_str).unwrap();
    let client = Client::builder().build()?;

    let balance = get_balance(&client, &stripe_token)?;
    let pending_amount = balance.pending.first().unwrap().amount;

    let charge = get_charge_by_payment(&successful_payment, &client, &stripe_token)?;
    let transfer_amount = charge.balance_transaction.net;

    // Check current available balance to be sure, that transfer-amount is covered
    if pending_amount > transfer_amount {
        let payment_intent = payment_intent_request(&client, &stripe_token, transfer_amount)?;
        let confirm_payment = confirm_payment(&payment_intent.id, &client, &stripe_token);
        match confirm_payment {
            Ok(confirmed) => set_transfer_id_on_payment(payment.id, &confirmed.id, &conn),
            Err(e) => eprintln!("Payment could not be transfered. Err: {}", e),
        }

        let reduced_balance = get_balance(&client, &stripe_token)?;
        let pending_amount_reduced = reduced_balance.pending.first().unwrap().amount;
        if pending_amount_reduced != pending_amount - transfer_amount {
            println!(
                "reduced_balance {:?} is NOT EQUAL to pending_amount {:?} - transfer_amount {:?}",
                pending_amount_reduced, pending_amount, transfer_amount
            );
        }
    }
    Ok(())
}

fn persist_payment(successful_payment: &SuccessfulPayment, conn: &db::UserDbConn) -> Payment {
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
    db::save_payment(new_payment, &conn)
}

fn payment_intent_request(
    client: &Client,
    token: &str,
    amount: i32,
) -> Result<PaymentIntent, reqwest::Error> {
    let destination_account = std::env::var("DESTINATION").unwrap();
    let payment_intent_forminfo = &[
        ("payment_method_types[]", "card"),
        ("amount", &amount.to_string()),
        ("currency", "eur"),
        ("transfer_data[destination]", &destination_account),
    ];

    client
        .post("https://api.stripe.com/v1/payment_intents")
        .bearer_auth(&token)
        .form(payment_intent_forminfo)
        .send()?
        .json::<PaymentIntent>()
}

fn confirm_payment(
    payment_id: &str,
    client: &Client,
    token: &str,
) -> Result<PaymentConfirmation, reqwest::Error> {
    let confirm_payment_endpoint = format!(
        "https://api.stripe.com/v1/payment_intents/{}/confirm",
        payment_id
    );
    client
        .post(&confirm_payment_endpoint)
        .bearer_auth(&token)
        // TODO: Use actual card-information
        .form(&[("payment_method", "pm_card_visa")])
        .send()?
        .json::<PaymentConfirmation>()
}

pub fn get_balance(client: &Client, token: &str) -> Result<Balance, reqwest::Error> {
    client
        .get("https://api.stripe.com/v1/balance")
        .bearer_auth(token)
        .send()?
        .json::<Balance>()
}

pub fn get_charge_by_payment(
    successful_payment: &SuccessfulPayment,
    client: &Client,
    token: &str,
) -> Result<ChargeResponse, reqwest::Error> {
    let charge_endpoint = format!(
        "https://api.stripe.com/v1/charges/{}",
        successful_payment.provider_payment_charge_id
    );
    client
        .get(&charge_endpoint)
        .bearer_auth(token)
        .form(&[("expand[]", "balance_transaction")])
        .send()?
        .json::<ChargeResponse>()
}

fn set_transfer_id_on_payment(payment_id: i32, transfer_id: &str, conn: &db::UserDbConn) {
    db::save_transfer_id(payment_id, transfer_id, conn);
}

// Helpers
pub fn money_in_eur(money: i64) -> f32 {
    money as f32 / 100.00
}

pub fn calc_stripe_fee(damage: i64) -> i32 {
    let fee_percentage = if is_test() { 0.029 } else { 0.014 };
    let fee_fix_amount = 0.25;
    let raw_damage = money_in_eur(damage);
    let total_damage = raw_damage * fee_percentage + fee_fix_amount;
    (total_damage * 100_f32) as i32
}
