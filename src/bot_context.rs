use crate::bot_types::{Keyboards, Payload, RequestType};
use crate::stripe_types::*;
use crate::models::{NewPayment, UpdateUser};
use crate::telegram_types::LabeledPrice as lp;
use crate::telegram_types::{self, *};
use crate::{db, messages, models};
use chrono::{DateTime, Duration, TimeZone, Utc};
use diesel::pg::data_types::PgTimestamp;
use diesel::pg::types::money::PgMoney;
use reqwest;
use reqwest::header::{self, HeaderMap};


// Everything higher than this cents value is forbidden
// to prevent the user from unintentionally high donations
pub static MAX_DAMAGE_ALLOWED: i64 = 1499;

pub struct BotContext {
    current_user: models::User,
    conn: db::UserDbConn,
    chat_id: i32,
    request_message: String,
    // Currently not used
    _date: DateTime<Utc>,
}

impl BotContext {
    pub fn new(
        current_user: models::User,
        conn: db::UserDbConn,
        chat_id: i32,
        request_message: String,
        timestamp: i64,
    ) -> Self {
        BotContext {
            current_user,
            conn,
            chat_id,
            request_message: request_message.to_string(),
            _date: Utc.timestamp(timestamp, 0),
        }
    }

    pub fn handle_request(
        &mut self,
        request_type: RequestType,
        keyboards: &Keyboards,
    ) -> serde_json::Result<String> {
        let response_text = match request_type {
            RequestType::Start => messages::WELCOME_MESSAGE.to_string(),
            RequestType::Terms => messages::TERMS.to_string(),
            RequestType::Order => {
                match self.order_drink() {
                    Some(new_drink_count) => format!("👍 Ich schreib's auf deinen Deckel.\n🍻 Bisher sind es {} Biers", new_drink_count),
                    None => format!("🤔 Du hast schon {:.2}€ auf dem Deckel.\n💰Der maximal erlaubte Schaden beträgt {:.2}€.\n💳 Ich muss leider erst abrechnen bevor du mehr bestellen kannst.", money_in_eur(self.get_damage()), money_in_eur(MAX_DAMAGE_ALLOWED)),
                } 

            }
            RequestType::ShowDamage => format!(
                "Du hast bisher {} Biers 🍻 bestellt.\nBeim aktuellen Preis 💶 von {:.2}€ beträgt dein derzeitiger Deckel insgesamt {:.2}€.",
                self.current_user.drink_count,
                money_in_eur(self.current_user.price.0),
                money_in_eur(self.get_damage())
            ),
            RequestType::BillPlease => format!(
                "💶 Dein derzeitiger Schaden beträgt {:.2}€. 💶\nMöchtest du wirklich zahlen?",
                money_in_eur(self.get_damage())
            ),
            RequestType::PayNo => "Ok, dann lass uns lieber weiter trinken.".to_string(),
            RequestType::DeletePlease => {
                "Möchtest du deine Userdaten wirklich löschen?".to_string()
            }
            RequestType::DeleteNo => "Ok, deine Daten wurden nicht gelöscht.".to_string(),
            RequestType::DeleteYes => {
                let deleted_user = self.delete_user();
                // TODO: delete returns count not deleted_user
                println!("User: {} with id: {} has been deleted", deleted_user, deleted_user);
                "No problemo. Ich habe deine Daten gelöscht.".to_string()
            }
            RequestType::Steal => {
                self.erase_drinks();
                "Ich habe deinen Deckel unauffällig zerrissen.".to_string()
            }
            RequestType::Options => "Was kann ich für dich tun?".to_string(),
            RequestType::ChangePrice => "Wähle einen neuen Getränkepreis.".to_string(),
            RequestType::NewPrice => {
                let new_price = self.convert_price();
                match self.update_price(new_price) {
                    Some(price) => {
                        format!(
                            "Alles klar, jedes Getränk kostet jetzt {:.2}€",
                            money_in_eur(price))
                    },
                    None => format!("Sorry, aber diese Erhöhung würde den zulässigen Schaden von {:.2}€ übersteigen.\nBitte zahle erst oder wähle einen anderen Preis.", money_in_eur(MAX_DAMAGE_ALLOWED)),
                }
            }
            RequestType::ShowLast => {
                let last_paid_amount = self.current_user.last_total.0;
                match last_paid_amount {
                    0 => "Du hast bisher noch nicht gespendet.".to_string(),
                    _ => format!("Deine letzte Spende war am {:?} und betrug {:.2}€.",
                        self.get_last_paid_as_date(), money_in_eur(self.current_user.last_total.0)),
                }
            }
            RequestType::ShowTotal => format!(
                "Insgesamt hast du {:.2}€ gespendet.",
                money_in_eur(self.current_user.total.0)
            ),
            RequestType::ShowTotalAll => {
                let total_all = self.get_total_all();
                match total_all {
                    0 => "Bisher wurde noch nicht gespendet".to_string(),
                    _ => format!("Zusammen haben wir bisher {:.2}€ gespendet.", money_in_eur(total_all)),
                }
            }
            RequestType::Unknown => {
                "🤷 Ehm, sorry darauf weiß ich grade keine Antwort...".to_string()
            }
            RequestType::PayYes => "IGNORED".to_string(),
        };

        match request_type {
            RequestType::PayYes => serde_json::to_string(&self.new_invoice()),
            _ => {
                let method = "sendMessage".to_string();
                let response_message = ResponseMessage::new(method, self.chat_id, response_text);
                let keyboard = keyboards.get_keyboard(request_type);
                let response_message = response_message.keyboard(keyboard);
                serde_json::to_string(&response_message)
            }
        }
    }

    pub fn order_drink(&mut self) -> Option<i16> {
        let new_drink_count = self.current_user.drink_count + 1;
        match (new_drink_count as i64 * self.current_user.price.0) < MAX_DAMAGE_ALLOWED {
            true => {
                let mut update_user = UpdateUser::from_user(&self.current_user);
                update_user.drink_count = Some(new_drink_count);
                db::update_user(self.current_user.id, &update_user, &self.conn);
                Some(new_drink_count)
            },
            false => None,
        }
    }

    pub fn get_damage(&self) -> i64 {
        let drinks = self.current_user.drink_count;
        let price = self.current_user.price.0;
        drinks as i64 * price
    }

    pub fn convert_price(&self) -> i64 {
        let new_price = self.request_message.replace("€", "").replace(",", "");
        let new_price = if &new_price[..1] == "0" {
            new_price[1..].to_string()
        } else {
            new_price
        };
        match new_price.parse::<i64>() {
            Ok(price) => price,
            Err(_) => unreachable!("Should not happen because, there is no RequestType::* for anything outside the button-options"),
        }
    }

    pub fn update_price(&mut self, new_price: i64) -> Option<i64> {
        let new_damage = self.current_user.drink_count as i64 * new_price;
        match new_damage < MAX_DAMAGE_ALLOWED {
            true => {
                let mut update_user = UpdateUser::from_user(&self.current_user);
                update_user.price = Some(PgMoney(new_price));
                db::update_user(self.current_user.id, &update_user, &self.conn);
                Some(new_price)
            },
            false => None,
        }
    }

    pub fn erase_drinks(&mut self) {
        let mut update_user = UpdateUser::from_user(&self.current_user);
        update_user.drink_count = Some(0);
        db::update_user(self.current_user.id, &update_user, &self.conn);
    }

    pub fn get_last_paid_as_date(&self) -> String {
        let date_time = Utc.timestamp(self.current_user.last_paid.0, 0);
        date_time.format("%d.%m.%Y um %H:%Mh").to_string()
    }

    pub fn get_total_all(&self) -> i64 {
        let vec_of_totals = db::get_total_all(&self.conn);
        vec_of_totals.iter().map(|money| money.0).sum()
    }

    pub fn delete_user(&self) -> usize {
        db::delete_user(&self.current_user, &self.conn)
    }

    pub fn get_request_type(
        &self,
        message: &telegram_types::Message,
        keyboards: &Keyboards,
    ) -> RequestType {
        let request_message = match &message.text {
            Some(text) => text.to_string(),
            None => match &message.sticker {
                Some(sticker) => match &sticker.emoji {
                    Some(emoji) => emoji.to_string(),
                    None => return RequestType::Unknown,
                },
                None => return RequestType::Unknown,
            },
        };
        if request_message == "/start" {
            return RequestType::Start;
        }
        if request_message == "/terms" {
            return RequestType::Terms;
        }
        keyboards.get_request_type(&request_message)
    }

    pub fn new_invoice(&self) -> InvoiceReplyMessage {
        let provider_token =
            std::env::var("PROVIDER_TOKEN").expect("Could not get provider_token from environment");
        let prices = vec![
            lp::new("Gesamt-Netto", self.get_damage_net()),
            lp::new("Stripe-Gebühr", self.stripe_fee()),
        ];
        let payload_result = serde_json::to_string(&Payload::new(
            self.current_user.id,
            self.chat_id,
            self.get_damage(),
            self.current_user.total.0,
        ));
        let payload = match payload_result {
            Ok(payload) => payload,
            Err(e) => panic!("could not parse payload. Error: {}", e),
        };
        InvoiceReplyMessage {
            method: "sendInvoice".to_string(),
            chat_id: self.chat_id,
            title: "Spendenrechnung".to_string(),
            description: format!(
                "TEST-Rechnung für eine Spende in Höhe von {:.2}€ an deine Lieblingskneipe.\n(Der Betrag enthält eine Gebühr von {:.2}€, der von dem Payment-Provider Stripe erhoben wird.) DIES IST EIN TEST!\nZAHLUNGEN SIND NOCH NICHT MÖGLICH!",
                money_in_eur(self.get_damage()),
                money_in_eur(self.stripe_fee() as i64),
            ),
            payload,
            provider_token,
            start_parameter: "TODO".to_string(),
            currency: "EUR".to_string(),
            prices,
            // provider_data: Some("TODO what does stripe need?".to_string()),
            provider_data: None,
            photo_url: Some("https://raw.githubusercontent.com/niilz/remoteDeckel/master/img/remoteDeckel-Logo.png".to_string()),
            // TODO: FIGURE OUT how to apply phot_size/width/height
            photo_size: 1,
            photo_width: 3,
            photo_height: 3,
            reply_markup: InlineKeyboardMarkup::new(money_in_eur(self.get_damage())),
        }
    }

    fn stripe_fee(&self) -> i32 {
        let fee_percentage = 0.014;
        let fee_fix_amount = 0.25;
        let raw_damage = money_in_eur(self.get_damage());
        let total_damage = raw_damage * fee_percentage + fee_fix_amount;
        (total_damage * 100_f32) as i32
    }

    fn get_damage_net(&self) -> i32 {
        self.get_damage() as i32 - self.stripe_fee()
    }
}

pub fn pay(successful_payment: &SuccessfulPayment, conn: db::UserDbConn) {
    // TODO: Do something with successful_payment information
    // And extract reply on successful_payment
    // And run pay!!! might have to be extracted from Bot_Context
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

pub fn transfer_money() -> Result<(), reqwest::Error> {
    let stripe_token = std::env::var("STRIPE_TOKEN_TEST").unwrap();
    // TODO: Check if money is actually on Stripe-account
    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(&stripe_token).unwrap());
    let client = reqwest::blocking::Client::builder()
        .build()?;
    let res = client
        .get("https://api.stripe.com/v1/balance")
        .bearer_auth(&stripe_token)
        .send();
    match res {
        Ok(res) => {
            println!("RES: {:?}", res);
            match res.json::<Balance>() {
                Ok(balance) => println!("BALANCE: {:?}", balance),
                Err(e) => eprintln!("Could not deserialize balance: {:?}", e),
            }
        },
        Err(e) => eprintln!("Request to Stripe did not work: {:?}", e),
    };
    // TODO: Make PaymentIntent
    // TODO: Confirm Payment
    Ok(())
}

// Helpers
pub fn money_in_eur(money: i64) -> f32 {
    money as f32 / 100.00
}
