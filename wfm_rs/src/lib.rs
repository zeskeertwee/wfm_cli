#![allow(unused_assignments)]
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub mod model;
pub mod request;
pub mod response;
pub mod shared;
pub mod traits;

pub use model::User;
pub use shared::Platform;

pub(crate) const BASE_URL: &str = "https://api.warframe.market/v1";

pub(crate) async fn get_endpoint<T: DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    jwt: &str,
) -> Result<T> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("authorization", jwt.parse()?);
    headers.insert("Content-Type", "application/json".parse()?);

    let raw = client
        .get(format!("{}{}", BASE_URL, url))
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    let base: response::ResponseWrapper<T> = serde_json::from_str(&raw)?;

    Ok(base.payload)
}

pub(crate) async fn get_endpoint_unauthorized<T: DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
) -> Result<T> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);

    let raw = client
        .get(format!("{}{}", BASE_URL, url))
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    let base: response::ResponseWrapper<T> = serde_json::from_str(&raw)?;

    Ok(base.payload)
}

pub(crate) async fn post_endpoint<T: DeserializeOwned, B: Serialize>(
    client: &reqwest::Client,
    url: &str,
    jwt: &str,
    body: &B,
) -> Result<T> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("authorization", jwt.parse()?);
    headers.insert("Content-Type", "application/json".parse()?);

    let raw = client
        .post(format!("{}{}", BASE_URL, url))
        .headers(headers)
        .body(serde_json::to_string(body)?)
        .send()
        .await?
        .text()
        .await?;

    let base: response::ResponseWrapper<T> = serde_json::from_str(&raw)?;

    Ok(base.payload)
}

pub(crate) async fn delete_endpoint<T: DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    jwt: &str,
) -> Result<T> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("authorization", jwt.parse()?);
    headers.insert("Content-Type", "application/json".parse()?);

    let raw = client
        .delete(format!("{}{}", BASE_URL, url))
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;

    let base: response::ResponseWrapper<T> = serde_json::from_str(&raw)?;

    Ok(base.payload)
}

pub(crate) async fn put_endpoint<T: Serialize>(
    client: &reqwest::Client,
    url: &str,
    jwt: &str,
    body: &T,
) -> Result<()> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("authorization", jwt.parse()?);
    headers.insert("Content-Type", "application/json".parse()?);

    let raw = client
        .put(format!("{}{}", BASE_URL, url))
        .headers(headers)
        .body(serde_json::to_string(body)?)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
