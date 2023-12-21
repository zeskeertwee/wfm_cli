use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Pc,
    Xbox,
    Ps4,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModPolarity {
    Madurai,
    Vazarin,
    Naramon,
    Zenurik,
}
