pub(crate) mod load_job;

use std::fmt::Formatter;
use std::io::{Read, Write};
use wfm_rs::response::{OrderUser, ShortItem};
use wfm_rs::shared::OrderType;
use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use crate::util::{create_storage_file, get_storage_file};

pub const LEDGER_KEY: &'static str = "__WMI_LEDGER";

#[derive(Serialize, Deserialize, Default)]
pub struct Ledger {
    transaction_id_counter: u64,
    pub transactions: Vec<Transaction>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct Transaction {
    pub transaction_id: TransactionId,
    pub item: TransactionItem,
    pub price: u32,
    pub kind: OrderType,
    pub recipient: TransactionUser,
    pub timestamp: DateTime<Local>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub enum TransactionItem {
    Item(TransactionShortItem),
    Other(String)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct TransactionUser {
    pub id: String,
    pub ingame_name: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct TransactionShortItem {
    pub id: String,
    pub item_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Hash)]
pub struct TransactionId(u64);

impl std::fmt::Display for TransactionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TRX-{}", self.0)
    }
}

impl Ledger {
    pub fn load_from_disk() -> Result<Self> {
        let mut file = get_storage_file("ledger.json")?;
        let mut str = String::new();
        file.read_to_string(&mut str)?;

        Ok(serde_json::from_str(&str)?)
    }

    pub fn save_to_disk(&self) -> Result<()> {
        let mut file = create_storage_file("ledger.json")?;
        let str = serde_json::to_string(&self)?;
        file.write_all(str.as_bytes())?;

        Ok(())
    }

    fn generate_transaction_id(&mut self) -> TransactionId {
        self.transaction_id_counter += 1;

        TransactionId(self.transaction_id_counter)
    }

    pub fn submit_transaction(&mut self, mut transaction: Transaction) {
        let id = self.generate_transaction_id();
        transaction.transaction_id = id;
        self.transactions.push(transaction);
    }
}

impl Transaction {
    pub fn new<I: Into<TransactionItem>, U: Into<TransactionUser>>(item: I, recipient: U, price: u32, kind: OrderType) -> Self {
        Self {
            item: item.into(),
            recipient: recipient.into(),
            price,
            kind,
            timestamp: Local::now(),
            // this is replaced when it's submitted to the ledger
            transaction_id: TransactionId(0)
        }
    }
}