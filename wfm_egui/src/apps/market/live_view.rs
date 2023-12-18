use eframe::egui::{Context, Ui};
use wfm_rs::websocket::{WebsocketMessagePayload, WebsocketOrder};
use wfm_rs::shared::OrderType;
use crate::app::{App, AppWindow, WFM_WS_EVENTS_KEY};

#[derive(Default)]
pub struct LiveMarketApp {
    buy_orders: Vec<WebsocketOrder>,
    sell_orders: Vec<WebsocketOrder>
}

impl AppWindow for LiveMarketApp {
    fn window_title(&self) -> String {
        "Live warframe.market orders".to_string()
    }

    fn update(&mut self, app: &App, ctx: &Context, ui: &mut Ui) {
        if app.present_in_storage(WFM_WS_EVENTS_KEY) {
            app.get_from_storage::<Vec<WebsocketMessagePayload>, _, _>(WFM_WS_EVENTS_KEY, |v| v.unwrap().iter().map(|v| match v {
                WebsocketMessagePayload::NewOrder { order } => match order.order_type {
                    OrderType::Buy => self.buy_orders.push(order.clone()),
                    OrderType::Sell => self.sell_orders.push(order.clone()),
                },
                _ => (),
            }).collect::<()>());
        }

        ui.label(format!("{} buy/{} sell orders", self.buy_orders.len(), self.sell_orders.len()));
    }
}