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
