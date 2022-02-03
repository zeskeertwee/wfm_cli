use crate::response::ProfileOrderResponseWrapper;
use crate::shared::OrderType;
use crate::traits::OrderID;
use crate::{
    delete_endpoint, get_endpoint, post_endpoint, put_endpoint, request, response, traits,
    Platform, BASE_URL,
};
use anyhow::Result;
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct User {
    client: reqwest::Client,
    jwt_token: String,
    username: String,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.username == other.username && self.jwt_token == other.jwt_token
    }
}

impl User {
    pub async fn login(
        email: &str,
        password: &str,
        platform: &Platform,
        language: &str,
    ) -> Result<User> {
        let post_body = request::Signin {
            auth_type: "header".into(),
            email: email.into(),
            password: password.into(),
        };

        let req_client = reqwest::Client::new();

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", "application/json; utf-8".parse()?);
        headers.insert("Authorization", "JWT".parse()?);
        headers.insert(
            "platform",
            format!("{:?}", platform).to_lowercase().parse()?,
        );
        headers.insert("language", language.parse()?);

        let raw_response = req_client
            .post(format!("{}/auth/signin", BASE_URL))
            .headers(headers)
            .body(serde_json::to_string(&post_body)?)
            .send()
            .await?;

        let jwt_token = match raw_response.headers().get("authorization") {
            Some(x) => x.clone(),
            None => anyhow::bail!("No jwt_token in headers!"),
        };

        let text = raw_response.text().await?;
        let response: response::ResponseWrapper<response::SigninWrapper> =
            serde_json::from_str(&text)?;

        return Ok(User {
            client: req_client,
            jwt_token: jwt_token.to_str()?.to_string(),
            username: response.payload.user.ingame_name,
        });
    }

    pub fn _from_jwt_token(jwt_token: &str, username: &str) -> User {
        User {
            jwt_token: jwt_token.to_string(),
            client: reqwest::Client::new(),
            username: username.to_string(),
        }
    }

    pub fn _jwt_token(&self) -> String {
        self.jwt_token.clone()
    }

    pub fn username(&self) -> String {
        self.username.clone()
    }

    pub async fn get_items(&self) -> Result<Vec<response::ShortItem>> {
        Ok(
            get_endpoint::<response::Items>(&self.client, "/items", &self.jwt_token)
                .await?
                .items,
        )
    }

    pub async fn get_item<T: traits::ItemUrl>(&self, item: &T) -> Result<response::LongItem> {
        get_endpoint::<response::LongItem>(
            &self.client,
            &format!("/items/{}", item.item_url()),
            &self.jwt_token,
        )
        .await
    }

    pub async fn get_item_orders<T: traits::ItemUrl>(
        &self,
        item: &T,
    ) -> Result<Vec<response::Order>> {
        Ok(get_endpoint::<response::Orders>(
            &self.client,
            &format!("/items/{}/orders", item.item_url()),
            &self.jwt_token,
        )
        .await?
        .orders)
    }

    pub async fn get_item_market_statistics<T: traits::ItemUrl>(
        &self,
        item: &T,
    ) -> Result<response::MarketStatisticsWrapper> {
        get_endpoint(
            &self.client,
            &format!("/items/{}/statistics", item.item_url()),
            &self.jwt_token,
        )
        .await
    }

    pub async fn get_user_orders(&self) -> Result<response::ExistingProfileOrders> {
        get_endpoint(
            &self.client,
            &format!("/profile/{}/orders", &self.username),
            &self.jwt_token,
        )
        .await
    }

    pub async fn post_order(
        &self,
        desc: &PostOrderDescriptor,
    ) -> Result<response::ProfileOrderResponse> {
        let body = request::ProfileOrder {
            item_id: desc.item_id.clone(),
            order_type: desc.kind.clone(),
            platinum: desc.price,
            quantity: desc.quantity,
            visible: desc.visible,
            rank: desc.rank,
            subtype: desc.subtype.clone(),
        };

        Ok(
            post_endpoint::<ProfileOrderResponseWrapper, request::ProfileOrder>(
                &self.client,
                "/profile/orders",
                &self.jwt_token,
                &body,
            )
            .await?
            .order,
        )
    }

    pub async fn remove_order<T: OrderID>(
        &self,
        order: &T,
    ) -> Result<response::RemoveOrderResponse> {
        Ok(delete_endpoint::<response::RemoveOrderResponse>(
            &self.client,
            &format!("/profile/orders/{}", order.order_id()),
            &self.jwt_token,
        )
        .await?)
    }

    pub async fn update_order<T: OrderID>(
        &self,
        order: &T,
        desc: &UpdateOrderDescriptor,
    ) -> Result<()> {
        let body = request::UpdateOrder {
            order_id: order.order_id().to_string(),
            platinum: desc.platinum,
            quantity: desc.quantity,
            visible: desc.visible,
            rank: desc.rank.clone(),
            subtype: desc.subtype.clone(),
        };

        Ok(put_endpoint(
            &self.client,
            &format!("/profile/orders/{}", order.order_id()),
            &self.jwt_token,
            &body,
        )
        .await?)
    }

    pub async fn get_auctions(&self) -> Result<response::ProfileAuctions> {
        Ok(get_endpoint(
            &self.client,
            &format!("/profile/{}/auctions", self.username),
            &self.jwt_token,
        )
        .await?)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PostOrderDescriptor {
    pub item_id: String,
    pub price: u64,
    pub kind: OrderType,
    pub visible: bool,
    pub quantity: u16,
    pub rank: Option<u8>,
    pub subtype: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateOrderDescriptor {
    pub platinum: u64,
    pub quantity: u16,
    pub visible: bool,
    pub rank: Option<u8>,
    pub subtype: Option<String>,
}
