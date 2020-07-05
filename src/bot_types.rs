use crate::bot_types::RequestType::*;
use crate::telegram_types::ReplyKeyboardMarkup;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum RequestType {
    Start,
    Terms,
    Order,
    ShowDamage,
    BillPlease,
    PayNo,
    PayYes,
    Steal,
    Options,
    DeletePlease,
    DeleteNo,
    DeleteYes,
    ChangePrice,
    NewPrice,
    ShowLast,
    ShowTotal,
    ShowTotalAll,
    Unknown,
}

pub struct Keyboards {
    pub main: Vec<(RequestType, String)>,
    pub pay: Vec<(RequestType, String)>,
    pub delete: Vec<(RequestType, String)>,
    pub options: Vec<(RequestType, String)>,
    pub price: Vec<(RequestType, String)>,
}
impl Keyboards {
    pub fn init() -> Self {
        let mut main = Vec::new();
        main.push((Order, "🍺 Bring mir ein Bier! 🍺".to_string()));
        main.push((ShowDamage, "😬 Was is mein Schaden? 😬".to_string()));
        main.push((BillPlease, "🙈 Augen zu und zahlen. 💶".to_string()));
        main.push((Options, "⚙ Optionen ⚙".to_string()));

        let mut pay = Vec::new();
        pay.push((PayYes, "✅ JA! Jetzt spenden ✅".to_string()));
        pay.push((PayNo, "❌ NEIN! Noch nicht spenden ❌".to_string()));
        pay.push((Steal, "👻 Zeche prellen... 🤫".to_string()));

        let mut delete = Vec::new();
        delete.push((DeleteYes, "✅ JA! Daten löschen ✅".to_string()));
        delete.push((DeleteNo, "❌ NEIN! Daten nicht löschen ❌".to_string()));

        let mut options = Vec::new();
        options.push((ChangePrice, "€ Preis ändern €".to_string()));
        options.push((ShowLast, "⌚ Meine letzte Spende ⌚".to_string()));
        options.push((ShowTotal, "➕ Summe meiner Spenden ➕".to_string()));
        options.push((ShowTotalAll, "➕➕Summe aller Spenden➕➕".to_string()));
        options.push((DeletePlease, "😱 Lösche meine Daten 😱".to_string()));

        let mut price = Vec::new();
        price.push((NewPrice, "0,50€".to_string()));
        price.push((NewPrice, "1,00€".to_string()));
        price.push((NewPrice, "1,50€".to_string()));
        price.push((NewPrice, "2,00€".to_string()));

        Keyboards {
            main,
            pay,
            delete,
            options,
            price,
        }
    }

    pub fn get_request_type(&self, user_answer: &str) -> RequestType {
        match get_request_type_by_answer(&self.main, user_answer) {
            Some(req_typ) => req_typ,
            None => match get_request_type_by_answer(&self.pay, user_answer) {
                Some(req_typ) => req_typ,
                None => match get_request_type_by_answer(&self.delete, user_answer) {
                    Some(req_typ) => req_typ,
                    None => match get_request_type_by_answer(&self.options, user_answer) {
                        Some(req_typ) => req_typ,
                        None => match get_request_type_by_answer(&self.price, user_answer) {
                            Some(req_typ) => req_typ,
                            None => RequestType::Unknown,
                        },
                    },
                },
            },
        }
    }

    pub fn get_keyboard(&self, request_type: RequestType) -> ReplyKeyboardMarkup {
        match request_type {
            RequestType::BillPlease => keyboard_factory(&self.pay),
            RequestType::DeletePlease => keyboard_factory(&self.delete),
            RequestType::Options => keyboard_factory(&self.options),
            RequestType::ChangePrice => keyboard_factory(&self.price),
            _ => keyboard_factory(&self.main),
        }
    }
}

fn get_request_type_by_answer(
    keyboard: &Vec<(RequestType, String)>,
    user_response: &str,
) -> Option<RequestType> {
    keyboard.into_iter().find_map(|(req_typ, button)| {
        if button == user_response {
            Some(*req_typ)
        } else {
            None
        }
    })
}

pub fn keyboard_factory(keyboard: &Vec<(RequestType, String)>) -> ReplyKeyboardMarkup {
    let keyboard: Vec<Vec<String>> = keyboard
        .iter()
        .map(|(_req_typ, button)| vec![button.to_string()])
        .collect();
    ReplyKeyboardMarkup {
        keyboard,
        resize_keyboard: true,
    }
}

#[derive(Debug, Serialize, Deserialize)]
// Will be send across the wire as String on InvoiceReplyMessage
// Final String can NOT be longer than 128 characters!!!
pub struct Payload {
    pub user_id: i32,
    pub chat_id: i32,
    pub total: i64,
    pub totals_sum: i64,
}

impl Payload {
    pub fn new(user_id: i32, chat_id: i32, total: i64, totals_sum: i64) -> Self {
        Payload {
            user_id,
            chat_id,
            total,
            totals_sum,
        }
    }
}
