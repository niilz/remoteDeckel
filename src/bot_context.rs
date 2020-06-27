use crate::bot_types::{keyboard_factory, Keyboards, RequestType};
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
    keyboards: Keyboards,
}

impl BotContext {
    pub fn new(
        current_user: models::User,
        conn: db::UserDbConn,
        chat_id: i32,
        request_message: String,
        timestamp: i64,
        keyboards: Keyboards,
    ) -> Self {
        BotContext {
            current_user,
            conn,
            chat_id,
            request_message: request_message.to_string(),
            date: Utc.timestamp(timestamp, 0),
            keyboards,
        }
    }

    pub fn handle_request(&mut self, request_type: RequestType) -> serde_json::Result<String> {
        let response_text = match request_type {
            RequestType::Start => messages::WELCOME_MESSAGE.to_string(),
            RequestType::Order => {
                self.order_drink();
                "👍 Ich schreib's auf deinen Deckel.".to_string()
            }
            RequestType::ShowDamage => format!(
                "Du hast bisher {} Biers bestellt.\nBeim aktuellen Preis von {:.2}€ beträgt dein derzeitiger Deckel insgesamt {:.2}€.",
                self.current_user.drink_count,
                self.money_in_eur(self.current_user.price.0),
                self.money_in_eur(self.get_damage())
            ),
            RequestType::BillPlease => format!(
                "💶 Dein derzeitiger Schaden beträgt {:.2}€. 💶\nMöchtest du wirklich zahlen?",
                self.money_in_eur(self.get_damage())
            ),
            RequestType::PayNo => "Ok, dann lass uns lieber weiter trinken.".to_string(),
            RequestType::PayYes => {
                format!(
                "🙏 Danke für deine Spende 🙏\n💶 in Höhe von {},-€ 💶\n🦸 Du bist ein Retter! 🦸",
                self.money_in_eur(self.get_damage()))
            }
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
                            self.money_in_eur(price)
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
                        self.get_last_paid_as_date(), self.money_in_eur(self.current_user.last_total.0)),
                }
            }
            RequestType::ShowTotal => format!(
                "Insgesamt hast du {:.2}€ gespendet.",
                self.money_in_eur(self.current_user.total.0)
            ),
            RequestType::ShowTotalAll => {
                let total_all = self.get_total_all();
                match total_all {
                    0 => "Bisher wurde noch nicht gespendet".to_string(),
                    _ => format!("Zusammen haben wir bisher {:.2}€ gespendet.", self.money_in_eur(total_all)),
                }
            }
            RequestType::Unknown => {
                "🤷 Ehm, sorry darauf weiß ich grade keine Antwort...".to_string()
            }
        };

        match request_type {
            RequestType::PayYes => {
                let invoice = self.new_invoice();
                let invoice_stringyfied = serde_json::to_string(&invoice);
                match invoice_stringyfied {
                    Ok(invoice) => {
                        println!("{:?}", invoice);
                        serde_json::to_string(&invoice)
                    }
                    // TODO: Propper ResponseError if invoice can not be created
                    Err(_e) => self.handle_request(RequestType::Unknown),
                }
            }
            _ => {
                let method = "sendMessage".to_string();
                let response_message = ResponseMessage::new(method, self.chat_id, response_text);
                let keyboard = self.provide_keyboard(request_type);
                let response_message = response_message.keyboard(keyboard);
                serde_json::to_string(&response_message)
            }
        }
    }

    pub fn order_drink(&mut self) {
        let mut update_user = UpdateUser::from_user(&self.current_user);
        update_user.drink_count = Some(self.current_user.drink_count + 1);
        db::update_user(&self.current_user, &update_user, &self.conn);
    }

    pub fn get_damage(&self) -> i64 {
        let drinks = self.current_user.drink_count;
        let price = self.current_user.price.0;
        drinks as i64 * price
    }

    pub fn money_in_eur(&self, money: i64) -> f32 {
        money as f32 / 100.00
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
        db::update_user(&self.current_user, &update_user, &self.conn);
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
        db::update_user(&self.current_user, &update_user, &self.conn);
    }

    pub fn erase_drinks(&mut self) {
        let mut update_user = UpdateUser::from_user(&self.current_user);
        update_user.drink_count = Some(0);
        db::update_user(&self.current_user, &update_user, &self.conn);
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

    pub fn get_request_type(&self, message: &telegram_types::Message) -> RequestType {
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
        self.keyboards.get_request_type(&request_message)
    }

    pub fn provide_keyboard(&self, request_type: RequestType) -> ReplyKeyboardMarkup {
        match request_type {
            RequestType::BillPlease => keyboard_factory(&self.keyboards.pay),
            RequestType::DeletePlease => keyboard_factory(&self.keyboards.delete),
            RequestType::Options => keyboard_factory(&self.keyboards.options),
            RequestType::ChangePrice => keyboard_factory(&self.keyboards.price),
            _ => keyboard_factory(&self.keyboards.main),
        }
    }
    pub fn new_invoice(&self) -> InvoiceReplyMessage {
        let provider_token =
            std::env::var("PROVIDER_TOKEN").expect("Could not get provider_token from environment");
        let user = &self.current_user;
        let prices = vec![
            lb::new("Getränkeanzahl", user.drink_count as i32),
            lb::new("Preis pro Getränk", user.price.0 as i32),
            lb::new("Gesamt", self.get_damage() as i32),
        ];
        InvoiceReplyMessage {
            method: "sendInvoice".to_string(),
            chat_id: self.chat_id,
            title: "Spendenrechnung".to_string(),
            description: format!(
                "Rechnung für eine Spende in Höhe von {:.2}€ an deine Lieblingskneipe",
                self.get_damage()
            ),
            payload: format!(
                "User with id: {} has received invoice over {:.2}€",
                user.id,
                self.get_damage()
            ),
            provider_token,
            start_parameter: "TODO".to_string(),
            currency: "EUR".to_string(),
            prices,
            // provider_data: Some("TODO what does stripe need?".to_string()),
            provider_data: None,
            // photo_url: Some("../img/remoteDeckel-Logo.png".to_string()),
            photo_url: None,
        }
    }
}
