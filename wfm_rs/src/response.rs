use crate::shared::{OrderType, Platform};
use crate::traits::{ItemID, ItemUrl, OrderID};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct SigninWrapper {
    pub user: Signin,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Signin {
    pub id: String,
    pub anonymous: bool,
    pub verification: bool,
    pub ingame_name: String,
    pub check_code: String,
    pub role: String,
    pub patreon_profile: Option<SigninPatreonProfile>,
    pub platform: Platform,
    pub region: String,
    pub banned: bool,
    pub ban_reason: Option<String>,
    pub avatar: Option<String>,
    pub background: Option<String>,
    pub linked_accounts: SigninLinkedAccounts,
    pub has_mail: bool,
    pub written_reviews: u64,
    pub unread_messages: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SigninPatreonProfile {
    pub patreon_founder: bool,
    pub subscription: bool,
    pub patreon_badge: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SigninLinkedAccounts {
    pub steam_profile: bool,
    pub patreon_profile: bool,
    pub xbox_profile: bool,
    pub discord_profile: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ResponseWrapper<T> {
    pub payload: T,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Items {
    pub items: Vec<ShortItem>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ShortItem {
    pub url_name: String,
    pub thumb: String,
    pub id: String,
    pub item_name: String,
}

impl ItemUrl for ShortItem {
    fn item_url(&self) -> &str {
        self.url_name.as_str()
    }
}

impl ItemID for ShortItem {
    fn item_id(&self) -> &str {
        self.id.as_str()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LongItem {
    pub id: String,
    pub items_in_set: Vec<LongSubItem>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LongSubItem {
    pub id: String,
    pub url_name: String,
    pub icon: String,
    pub thumb: String,
    pub sub_icon: String,
    pub mod_max_rank: u8,
    pub subtypes: Vec<String>,
    pub tags: Vec<String>,
    pub cyan_stars: u16,
    pub amber_stars: u16,
    pub ducats: u16,
    pub set_root: bool,
    pub mastery_rank: u8,
    pub rarity: String,
    pub trading_tax: u32,
    pub en: LongSubItemLocale,
    pub ru: LongSubItemLocale,
    pub ko: LongSubItemLocale,
    pub fr: LongSubItemLocale,
    pub sv: LongSubItemLocale,
    pub de: LongSubItemLocale,
    #[serde(rename = "zh-hant")]
    pub zh_hant: LongSubItemLocale,
    #[serde(rename = "zh-hans")]
    pub zh_hans: LongSubItemLocale,
    pub pt: LongSubItemLocale,
    pub es: LongSubItemLocale,
    pub pl: LongSubItemLocale,
}

impl ItemUrl for LongSubItem {
    fn item_url(&self) -> &str {
        self.url_name.as_str()
    }
}

impl ItemID for LongSubItem {
    fn item_id(&self) -> &str {
        self.id.as_str()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LongSubItemLocale {
    pub item_name: String,
    pub description: String,
    pub wiki_link: String,
    pub drop: Vec<ItemDrop>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ItemDrop {
    pub name: String,
    pub link: String,
}

impl ItemUrl for ItemDrop {
    fn item_url(&self) -> &str {
        self.link.as_str()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Orders {
    pub orders: Vec<Order>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Order {
    pub quantity: u16,
    pub user: OrderUser,
    pub visible: bool,
    // TODO: parse these to timestamps
    pub last_update: String,
    pub creation_date: String,
    /// Sometimes this is a floating-point number
    pub platinum: f64,
    pub platform: Platform,
    pub order_type: OrderType,
    pub region: String,
    pub id: String,
}

impl OrderID for Order {
    fn order_id(&self) -> &str {
        self.id.as_str()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OrderUser {
    pub reputation: u16,
    pub region: String,
    pub ingame_name: String,
    pub status: String,
    pub id: String,
    // missing: avatar, last_seen
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MarketStatisticsWrapper {
    pub statistics_closed: MarketStatistics<MarketSubStatisticsClosed>,
    pub statistics_live: MarketStatistics<MarketSubStatisticsLive>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MarketStatistics<T> {
    #[serde(rename = "48hours")]
    pub _48_hours: Vec<T>,
    #[serde(rename = "90days")]
    pub _90_days: Vec<T>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MarketSubStatisticsClosed {
    pub datetime: String,
    pub volume: f32,
    pub min_price: f32,
    pub max_price: f32,
    pub open_price: f32,
    pub closed_price: f32,
    pub avg_price: f32,
    pub wa_price: f32,
    pub median: f32,
    pub donch_top: f32,
    pub donch_bot: f32,
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MarketSubStatisticsLive {
    pub datetime: String,
    pub volume: f32,
    pub min_price: f32,
    pub max_price: f32,
    pub avg_price: f32,
    pub wa_price: f32,
    pub median: f32,
    pub order_type: String,
    pub moving_avg: Option<f32>,
    pub id: String,
}

/// received from placing an order
#[derive(Serialize, Deserialize, Clone)]
pub struct ProfileOrderResponseWrapper {
    pub order: ProfileOrderResponse,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProfileOrderResponse {
    pub id: String,
    pub platinum: f64,
    pub quantity: u16,
    pub order_type: OrderType,
    pub platform: Platform,
    pub region: String,
    pub creation_date: String,
    pub last_update: String,
    pub visible: bool,
}

impl OrderID for ProfileOrderResponse {
    fn order_id(&self) -> &str {
        self.id.as_str()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProfileOrderItem {
    pub id: String,
    pub url_name: String,
    pub icon: String,
    pub thumb: String,
    pub sub_icon: String,
    pub mod_max_rank: u8,
    pub subtypes: Vec<String>,
    pub tags: Vec<String>,
    pub cyan_stars: u16,
    pub amber_stars: u16,
    pub ducats: u16,
    pub en: ShortItemLocale,
    pub ru: ShortItemLocale,
    pub ko: ShortItemLocale,
    pub fr: ShortItemLocale,
    pub sv: ShortItemLocale,
    pub de: ShortItemLocale,
    #[serde(rename = "zh-hant")]
    pub zh_hant: ShortItemLocale,
    #[serde(rename = "zh-hans")]
    pub zh_hans: ShortItemLocale,
    pub pt: ShortItemLocale,
    pub es: ShortItemLocale,
    pub pl: ShortItemLocale,
}

impl ItemUrl for ProfileOrderItem {
    fn item_url(&self) -> &str {
        self.url_name.as_str()
    }
}

impl ItemID for ProfileOrderItem {
    fn item_id(&self) -> &str {
        self.id.as_str()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ShortItemLocale {
    pub item_name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ExistingProfileOrders {
    pub sell_orders: Vec<ExistingProfileOrder>,
    pub buy_orders: Vec<ExistingProfileOrder>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ExistingProfileOrder {
    pub quantity: u16,
    pub last_update: String,
    pub platinum: f64,
    pub order_type: OrderType,
    pub region: String,
    pub item: ExistingProfileOrderItem,
    /// this is the order id
    pub id: String,
    pub platform: Platform,
    pub creation_date: String,
    pub visible: bool,
}

impl OrderID for ExistingProfileOrder {
    fn order_id(&self) -> &str {
        self.id.as_str()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ExistingProfileOrderItem {
    pub quantity_for_set: Option<u16>,
    pub ducats: Option<u16>,
    pub id: String,
    pub url_name: String,
    pub icon: String,
    pub thumb: String,
    pub icon_format: String,
    pub tags: Vec<String>,
    pub sub_icon: Option<String>,
    pub en: ShortItemLocale,
    pub ru: ShortItemLocale,
    pub ko: ShortItemLocale,
    pub fr: ShortItemLocale,
    pub sv: ShortItemLocale,
    pub de: ShortItemLocale,
    #[serde(rename = "zh-hant")]
    pub zh_hant: ShortItemLocale,
    #[serde(rename = "zh-hans")]
    pub zh_hans: ShortItemLocale,
    pub pt: ShortItemLocale,
    pub es: ShortItemLocale,
    pub pl: ShortItemLocale,
}

impl ItemUrl for ExistingProfileOrderItem {
    fn item_url(&self) -> &str {
        self.url_name.as_str()
    }
}

impl ItemID for ExistingProfileOrderItem {
    fn item_id(&self) -> &str {
        self.id.as_str()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RemoveOrderResponse {
    pub order_id: String,
}

impl OrderID for RemoveOrderResponse {
    fn order_id(&self) -> &str {
        self.order_id.as_str()
    }
}
