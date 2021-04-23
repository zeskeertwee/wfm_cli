use serde::{Serialize, Deserialize};
use crate::traits::ItemUrl;

#[derive(Serialize, Deserialize, Clone)]
pub struct Signin {
    pub id: String,
    pub anonymous: bool,
    pub verification: bool,
    pub ingame_name: String,
    pub check_role: String,
    pub role: String,
    pub patreon_profile: SigninPatreonProfile,
    pub platform: String,
    pub region: String,
    pub banned: bool,
    pub ban_reason: String,
    pub avatar: String,
    pub background: String,
    pub linked_accounts: SigninLinkedAccounts,
    pub has_email: bool,
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
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ResponseWrapper<T> {
    pub(crate) payload: T,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Items {
    pub items: Vec<ShortItem>
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
    pub platform: String,
    pub order_type: String,
    pub region: String,
    pub id: String,
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