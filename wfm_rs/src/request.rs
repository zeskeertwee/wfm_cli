use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Signin {
    pub(crate) auth_type: String,
    pub(crate) email: String,
    pub(crate) password: String,
}