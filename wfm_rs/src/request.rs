use crate::shared::OrderType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Signin {
    pub(crate) auth_type: String,
    pub(crate) email: String,
    pub(crate) password: String,
}

/// used to place an order
#[derive(Serialize, Deserialize)]
pub struct ProfileOrder {
    pub item_id: String,
    pub order_type: OrderType,
    pub platinum: u64,
    pub quantity: u16,
    pub visible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateOrder {
    pub item_id: String,
    pub platinum: u64,
    pub quantity: u16,
    pub visible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,
}
