use crate::bot_types::RequestType::*;
use crate::telegram_types::ReplyKeyboardMarkup;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum RequestType {
    Start,
    Order,
    ShowDamage,
    BillPlease,
    PayNo,
    PayYes,
    Options,
    DeletePlease,
    DeleteNo,
    DeleteYes,
    ChangePrice,
    ShowLast,
    ShowTotal,
    ShowTotalAll,
    SmallPrice,
    MiddlePrice,
    HighPrice,
    PremiumPrice,
    Unknown,
}

pub struct Keyboards {
    pub main: HashMap<RequestType, Vec<String>>,
    pub pay: HashMap<RequestType, Vec<String>>,
    pub delete: HashMap<RequestType, Vec<String>>,
    pub options: HashMap<RequestType, Vec<String>>,
    pub price: HashMap<RequestType, Vec<String>>,
}
impl Keyboards {
    pub fn init() -> Self {
        let mut main = HashMap::new();
        main.insert(Order, vec!["🍺 Bring mir ein Bier! 🍺".to_string()]);
        main.insert(ShowDamage, vec!["😬 Was is mein Schaden? 😬".to_string()]);
        main.insert(BillPlease, vec!["🙈 Augen zu und zahlen. 💶".to_string()]);
        main.insert(Options, vec!["⚙ Optionen ⚙".to_string()]);

        let mut pay = HashMap::new();
        pay.insert(PayYes, vec!["✅ JA! Jetzt spenden ✅".to_string()]);
        pay.insert(PayNo, vec!["❌ NEIN! Noch nicht spenden ❌".to_string()]);

        let mut delete = HashMap::new();
        delete.insert(DeleteYes, vec!["✅ JA! Daten löschen ✅".to_string()]);
        delete.insert(
            DeleteNo,
            vec!["❌ NEIN! Daten nicht löschen ❌".to_string()],
        );

        let mut options = HashMap::new();
        options.insert(ChangePrice, vec!["€ Preis ändern €".to_string()]);
        options.insert(ShowLast, vec!["⌚ Meine letzte Spende ⌚".to_string()]);
        options.insert(ShowTotal, vec!["➕ Summe meiner Spenden ➕".to_string()]);
        options.insert(
            ShowTotalAll,
            vec!["➕➕Summe aller Spenden➕➕".to_string()],
        );
        options.insert(DeletePlease, vec!["😱 Lösche meine Daten 😱".to_string()]);

        let mut price = HashMap::new();
        price.insert(SmallPrice, vec!["0,50€".to_string()]);
        price.insert(MiddlePrice, vec!["1.00€".to_string()]);
        price.insert(HighPrice, vec!["1.50€".to_string()]);
        price.insert(PremiumPrice, vec!["2.00€".to_string()]);

        Keyboards {
            main,
            pay,
            delete,
            options,
            price,
        }
    }

    pub fn get_request_type(&self, user_answer: &str) -> RequestType {
        match find_key_by_value(&self.main, user_answer) {
            Some(req_typ) => req_typ,
            None => match find_key_by_value(&self.pay, user_answer) {
                Some(req_typ) => req_typ,
                None => match find_key_by_value(&self.delete, user_answer) {
                    Some(req_typ) => req_typ,
                    None => match find_key_by_value(&self.options, user_answer) {
                        Some(req_typ) => req_typ,
                        None => match find_key_by_value(&self.price, user_answer) {
                            Some(req_typ) => req_typ,
                            None => RequestType::Unknown,
                        },
                    },
                },
            },
        }
    }
}

fn find_key_by_value(
    map: &HashMap<RequestType, Vec<String>>,
    user_response: &str,
) -> Option<RequestType> {
    map.iter().find_map(|(req_typ, button)| {
        if button[0] == user_response {
            Some(*req_typ)
        } else {
            None
        }
    })
}

pub fn keyboard_factory(keyboard_map: &HashMap<RequestType, Vec<String>>) -> ReplyKeyboardMarkup {
    let keyboard: Vec<Vec<String>> = keyboard_map
        .iter()
        .map(|(_req_typ, buttons)| buttons.iter().map(|button| button.to_string()).collect())
        .collect();
    ReplyKeyboardMarkup {
        keyboard,
        resize_keyboard: false,
    }
}
