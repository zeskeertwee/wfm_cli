use reqwest;
use anyhow::Result;
use crate::{ request, response, traits, BASE_URL, get_endpoint };

pub struct User {
    client: reqwest::Client,
    jwt_token: String,
}

impl User {
    pub async fn login(email: &str, password: &str, platform: &str, language: &str) -> Result<User> {
        let post_body = request::Signin {
            auth_type: "header".into(),
            email: email.into(),
            password: password.into(),
        };

        let req_client = reqwest::Client::new();

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", "application/json; utf-8".parse()?);
        headers.insert("Authorization", "JWT".parse()?);
        headers.insert("platform", platform.parse()?);
        headers.insert("language", language.parse()?);

        let raw_response = req_client.post(format!("{}/auth/signin", BASE_URL))
            .headers(headers)
            .body(serde_json::to_string(&post_body)?)
            .send()
            .await?;

        
        let jwt_token = match raw_response.headers().get("authorization") {
            Some(x) => x,
            None => anyhow::bail!("No jwt_token in headers!"),
        };

        return Ok(User {
            client: req_client,
            jwt_token: jwt_token.to_str()?.to_string(),
        });
    }

    pub fn _from_jwt_token(jwt_token: &str) -> User {
        User {
            jwt_token: jwt_token.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn _jwt_token(&self) -> String {
        self.jwt_token.clone()
    }

    pub async fn get_items(&self) -> Result<Vec<response::ShortItem>> {
        Ok(get_endpoint::<response::Items>(&self.client, "/items", &self.jwt_token).await?.items)
    }

    pub async fn get_item<T: traits::ItemUrl>(&self, item: &T) -> Result<response::LongItem> {
        get_endpoint::<response::LongItem>(&self.client, &format!("/items/{}", item.item_url()), &self.jwt_token).await
    }

    pub async fn get_item_orders<T: traits::ItemUrl>(&self, item: &T) -> Result<Vec<response::Order>> {
        Ok(get_endpoint::<response::Orders>(&self.client, &format!("/items/{}/orders", item.item_url()), &self.jwt_token).await?.orders)
    }

    pub async fn get_item_market_statistics<T: traits::ItemUrl>(&self, item: &T) -> Result<response::MarketStatisticsWrapper> {
        get_endpoint(&self.client, &format!("/items/{}/statistics", item.item_url()), &self.jwt_token).await
    }
}