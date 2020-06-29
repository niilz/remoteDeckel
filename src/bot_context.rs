use crate::bot_types::{keyboard_factory, Keyboards, Payload, RequestType};
use crate::models::UpdateUser;
use crate::telegram_types::LabeledPrice as lb;
use crate::telegram_types::{self, *};
use crate::{db, messages, models};
use chrono::{DateTime, TimeZone, Utc};
use diesel::pg::data_types::PgTimestamp;
use diesel::pg::types::money::PgMoney;

pub struct BotContext {
    current_user: models::User,
    conn: db::UserDbConn,
    chat_id: i32,
    request_message: String,
    date: DateTime<Utc>,
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
            date: Utc.timestamp(timestamp, 0),
        }
    }

    pub fn handle_request(
        &mut self,
        request_type: RequestType,
        keyboards: &Keyboards,
    ) -> serde_json::Result<String> {
        let response_text = match request_type {
            RequestType::Start => messages::WELCOME_MESSAGE.to_string(),
            RequestType::Order => {
                if self.get_damage() > 1000 {
                    format!("🤔 Du hast schon {:.2}€ auf dem Deckel.
                        Ich muss leider erst abrechnen bevor du mehr bestellen kannst.", money_in_eur(self.get_damage()))
                } else {
                    self.order_drink();
                    "👍 Ich schreib's auf deinen Deckel.".to_string()
                }
            }
            RequestType::ShowDamage => format!(
                "Du hast bisher {} Biers bestellt.\nBeim aktuellen Preis von {:.2}€ beträgt dein derzeitiger Deckel insgesamt {:.2}€.",
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
                match new_price {
                    Ok(price) if price <= 200 => {
                        self.update_price(price);
                        format!(
                            "Alles klar, jedes Getränk kostet jetzt {:.2}€",
                            money_in_eur(price)
                        )
                    }
                    _ => "🥁 Sorry aber das ist hier kein Wunschkonzert 🥁".to_string(),
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

    pub fn order_drink(&mut self) {
        let mut update_user = UpdateUser::from_user(&self.current_user);
        update_user.drink_count = Some(self.current_user.drink_count + 1);
        db::update_user(self.current_user.id, &update_user, &self.conn);
    }

    pub fn get_damage(&self) -> i64 {
        let drinks = self.current_user.drink_count;
        let price = self.current_user.price.0;
        drinks as i64 * price
    }

    pub fn convert_price(&self) -> Result<i64, std::num::ParseIntError> {
        let new_price = self.request_message.replace("€", "").replace(",", "");
        let new_price = if &new_price[..1] == "0" {
            new_price[1..].to_string()
        } else {
            new_price
        };
        new_price.parse::<i64>()
    }

    pub fn update_price(&mut self, new_price: i64) {
        let mut update_user = UpdateUser::from_user(&self.current_user);
        update_user.price = Some(PgMoney(new_price));
        db::update_user(self.current_user.id, &update_user, &self.conn);
    }

    pub fn pay(&mut self) {
        let last_paid = self.date.timestamp();
        let new_last_total = self.get_damage();
        let total = self.current_user.total.0 + new_last_total;
        let mut update_user = UpdateUser::default();
        update_user.last_paid = Some(PgTimestamp(last_paid));
        update_user.last_total = Some(PgMoney(new_last_total));
        update_user.total = Some(PgMoney(total));
        update_user.drink_count = Some(0);
        db::update_user(self.current_user.id, &update_user, &self.conn);
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
        keyboards.get_request_type(&request_message)
    }

    pub fn new_invoice(&self) -> InvoiceReplyMessage {
        let provider_token =
            std::env::var("PROVIDER_TOKEN").expect("Could not get provider_token from environment");
        let prices = vec![
            lb::new("Gesamt-Netto", self.get_damage_net()),
            lb::new("Stripe-Gebühr", self.stripe_fee()),
        ];
        let payload = match serde_json::to_string(&Payload::new(
            &self.current_user,
            self.chat_id,
            self.get_damage(),
        )) {
            Ok(payload) => payload,
            Err(_) => "Could not parse the users payload".to_string(),
        };
        InvoiceReplyMessage {
            method: "sendInvoice".to_string(),
            chat_id: self.chat_id,
            title: "Spendenrechnung".to_string(),
            description: format!(
                "Rechnung für eine Spende in Höhe von {:.2}€ an deine Lieblingskneipe.\n(Der Betrag enthält eine Gebühr von {:.2}€, der von dem Payment-Provider Stripe erhoben wird.)",
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
            // photo_url: Some("../img/remoteDeckel-Logo.png".to_string()),
            photo_url: None,
            // TODO: change inline_keyboard_markup to "spenden" instead of "bezahlen".
            // If possible change the three option below (without PayYes but with Steal and PayNo)
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

// Payment is not on bot_context because we have to perform it after successful_payment, where we
// don't have the user infos in the typical way.
pub fn pay(payload: &Payload, conn: db::UserDbConn) {
    let last_paid = payload.payed_at;
    let new_last_total = payload.total;
    let total = payload.totals_sum + new_last_total;
    let mut update_user = UpdateUser::default();
    update_user.last_paid = Some(PgTimestamp(last_paid));
    update_user.last_total = Some(PgMoney(new_last_total));
    update_user.total = Some(PgMoney(total));
    update_user.drink_count = Some(0);
    db::update_user(payload.user_id, &update_user, &conn);
}

// Helpers
pub fn money_in_eur(money: i64) -> f32 {
    money as f32 / 100.00
}
