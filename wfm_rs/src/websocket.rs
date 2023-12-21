use std::net::TcpStream;
use anyhow::bail;
use serde_json;
use tungstenite;
use serde::{Serialize, Deserialize};
use crate::{Platform, User};
use tungstenite::http::Request;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};
use log::{error, info, trace};
use tungstenite::client::IntoClientRequest;
use tungstenite::Error::Utf8;
use crate::response::{ExistingProfileOrderItem, OrderUser, ShortItemLocale};
use crate::shared::OrderType;

// TODO: un-hardcode platform
const WS_URL: &'static str = "wss://warframe.market/socket?platform=pc";

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Online,
    Offline,
    InGame
}

#[derive(Serialize, Deserialize)]
pub struct WebsocketMessage {
    pub r#type: String,
    #[serde(skip_serializing_if = "serde_should_serialize_payload")]
    pub payload: WebsocketMessagePayload
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum WebsocketMessagePayload {
    // sent both ways
    #[serde(rename = "@WS/USER/SET_STATUS")]
    SetStatus(Status),
    // sent to server
    #[serde(rename = "@WS/SUBSCRIBE/MOST_RECENT")]
    SubscribeMostRecent,
    // sent from server
    #[serde(rename = "@WS/MESSAGE/ONLINE_COUNT")]
    OnlineCount(OnlineCount),
    #[serde(rename = "@WS/SUBSCRIPTIONS/MOST_RECENT/NEW_ORDER")]
    NewOrder {
        order: WebsocketOrder
    },
    #[serde(rename = "@WS/ERROR")]
    Error(String)
}

fn serde_should_serialize_payload(payload: &WebsocketMessagePayload) -> bool {
    payload == &WebsocketMessagePayload::SubscribeMostRecent
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct OnlineCount {
    pub total_users: u32,
    pub registered_users: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct WebsocketOrder {
    pub id: String,
    pub quantity: u16,
    pub user: OrderUser,
    pub platform: Platform,
    pub order_type: OrderType,
    pub item: ExistingProfileOrderItem,
    pub creation_date: String,
    pub visible: bool,
    pub last_update: String,
    pub platinum: f64,
    pub region: String,
}

impl WebsocketMessagePayload {
    pub fn get_ws_type(&self) -> String {
        match self {
            WebsocketMessagePayload::SetStatus(_) => "@WS/USER/SET_STATUS",
            WebsocketMessagePayload::SubscribeMostRecent => "@WS/SUBSCRIBE/MOST_RECENT",
            WebsocketMessagePayload::OnlineCount(_) => "@WS/MESSAGE/ONLINE_COUNT",
            WebsocketMessagePayload::NewOrder { order: _order } => "@WS/SUBSCRIPTIONS/MOST_RECENT/NEW_ORDER",
            WebsocketMessagePayload::Error(_) => "@WS/ERROR"
        }.to_string()
    }
}

pub struct WebsocketConnection {
    socket: WebSocket<MaybeTlsStream<TcpStream>>
}

impl WebsocketConnection {
    pub fn create(user: Option<&User>) -> anyhow::Result<Self> {
        info!("Opening WS connection to {}", WS_URL);

        let mut req = WS_URL.into_client_request()?;
        if let Some(user) = user {
            req.headers_mut()
                .insert("Cookie", format!("JWT={}", user._jwt_token().replace("JWT ", "")).parse()?);
        }
        
        let (socket, _resp) = match tungstenite::connect(req) {
            Ok(v) => v,
            Err(e) => {
                match &e {
                    tungstenite::Error::Http(resp) => {
                        error!("Result body: {:?}", resp.body().as_ref().map(|v| String::from_utf8(v.to_vec())));
                    },
                    _ => (),
                }

                error!("Failed to open WS connection: {:?}", e);
                return Err(e)?;
            }
        };

        Ok(Self {
            socket
        })
    }

    pub fn send_message(&mut self, msg: WebsocketMessagePayload) -> anyhow::Result<()> {
        let text = serde_json::to_string(&msg)?;
        trace!("Sending payload: {}", text);
        self.socket.send(Message::Text(text))?;
        Ok(())
    }

    pub fn read_message(&mut self) -> anyhow::Result<WebsocketMessagePayload> {
        match self.socket.read()? {
            Message::Text(text) => {
                trace!("Received WS message: {}", text);
                let msg: WebsocketMessagePayload = serde_json::from_str(&text)?;
                return Ok(msg);
            },
            x => trace!("Ignored WS message: {:?}", x),
        };

        bail!("Failed to receive a message or received unexpected message type");
    }
}