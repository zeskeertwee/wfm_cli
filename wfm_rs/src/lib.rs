#![allow(unused_assignments)]
use serde::de::DeserializeOwned;
use anyhow::Result;

pub mod response;
pub mod request;
pub mod model;
pub mod traits;

pub use model::User;

pub(crate) const BASE_URL: &str = "https://api.warframe.market/v1";

pub(crate) async fn get_endpoint<T: DeserializeOwned>(client: &reqwest::Client, url: &str, jwt: &str) -> Result<T> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("authorization", jwt.parse()?);
    headers.insert("Content-Type", "application/json".parse()?);

    let raw = client.get(format!("{}{}", BASE_URL, url))
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    let base: response::ResponseWrapper<T> = serde_json::from_str(&raw)?;

    Ok(base.payload)
}