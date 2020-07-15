use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Balance {
    pub available: Vec<Fund>,
    pub connect_reserved: Vec<Fund>,
    pub livemode: bool,
    pub pending: Vec<Fund>,
}

#[derive(Debug, Deserialize)]
pub struct Fund {
    pub amount: i32,
    pub currency: String,
    pub source_types: Option<SourceType>,
}

#[derive(Debug, Deserialize)]
pub struct SourceType {
    pub card: i32,
}

#[derive(Debug, Deserialize)]
pub struct PaymentIntent {
    pub id: String,
    pub amount: i32,
    pub amount_received: i32,
    pub currency: String,
    pub application_fee_amount: Option<i32>,
    pub created: i64,
}

#[derive(Debug, Deserialize)]
pub struct PaymentConfirmation {
    pub id: String,
    pub amount: i32,
    pub amount_received: i32,
    pub application: Option<String>,
    pub charges: Charge,
    pub confirmation_method: String,
    pub created: i64,
    pub currency: String,
    pub description: Option<String>,
    pub metadata: Option<MetaData>,
    pub payment_method: String,
    pub payment_method_types: Vec<String>,
    pub setup_future_usage: Option<String>,
    pub statement_descriptor: Option<String>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct Charge {
    data: Vec<Data>,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub id: String,
    pub amount: i32,
    pub amount_refunded: i32,
    pub application_fee_amount: Option<i32>,
    pub balance_transaction: String,
    pub billing_details: BillingDetails,
    pub captured: bool,
    pub created: i64,
    pub currency: String,
    pub description: Option<String>,
    pub disputed: bool,
    pub failure_message: Option<String>,
    pub fraud_details: Option<FraudDetails>,
    pub metadata: Option<MetaData>,
    pub outcome: Outcome,
    pub paid: bool,
    pub payment_intent: String,
    pub payment_method: String,
    // TODO: Think about more fields, that could be reasonable to deserialize
}

#[derive(Debug, Deserialize)]
pub struct BillingDetails {
    pub address: Address,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Address {
    pub country: Option<String>,
    pub postal_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MetaData {
    pub tguser: Option<String>,
    pub payload: Option<String>,
    pub bot: Option<String>,
    pub callback: Option<String>,
    pub tgcharge_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Outcome {
    pub network_status: String,
    pub reason: Option<String>,
    pub risk_level: String,
    pub risk_score: i32,
    pub seller_message: String,
    #[serde(rename(deserialize = "type"))]
    pub typ: String,
}

#[derive(Debug, Deserialize)]
pub struct FraudDetails {}

#[derive(Debug, Deserialize)]
pub struct ChargeResponse {
    pub id: String,
    pub amount: i32,
    pub amount_refunded: i32,
    pub balance_transaction: BalanceTransaction,
}

#[derive(Debug, Deserialize)]
pub struct BalanceTransaction {
    pub id: String,
    pub amount: i32,
    pub available_on: i64,
    pub created: i64,
    pub currency: String,
    pub fee: i32,
    pub fee_details: FeeDetails,
    pub net: i32,
    pub reporting_category: String,
    pub status: String,
    #[serde(rename(deserialize = "type"))]
    pub typ: String,
}

#[derive(Debug, Deserialize)]
pub struct FeeDetails {
    pub amount: i32,
    pub description: String,
    #[serde(rename(deserialize = "type"))]
    pub typ: String,
    pub destination: String,
}
