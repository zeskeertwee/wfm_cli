use crate::app::{App, AppWindow};
use crate::background_jobs::wfm_manifest::WarframeMarketManifest;
use crate::background_jobs::wfm_profile_orders::WFM_EXISTING_PROFILE_ORDERS_KEY;
use eframe::egui::{CtxRef, ScrollArea, Ui};
use wfm_rs::response::ExistingProfileOrders;

#[derive(Default)]
pub struct ExistingProfileOrdersApp {
    orders: Option<ExistingProfileOrders>,
}

impl AppWindow for ExistingProfileOrdersApp {
    fn window_title(&self) -> &str {
        "Warframe.market orders"
    }

    fn update(&mut self, app: &App, ctx: &CtxRef, ui: &mut Ui) {
        if self.orders.is_none() {
            match app.get_from_storage::<ExistingProfileOrders, _, _>(
                WFM_EXISTING_PROFILE_ORDERS_KEY,
                |orders| orders.and_then(|orders| Some(orders.clone())),
            ) {
                Some(orders) => {
                    self.orders = Some(orders);
                }
                None => {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        ui.label("Warframe.market item manifest not loaded!");
                    });
                    ui.add_space(40.0);
                    return;
                }
            }
        }

        let orders = self.orders.as_ref().unwrap();

        ScrollArea::new([false, true]).show(ui, |ui| {
            for order in orders.sell_orders.iter() {
                ui.label(format!(
                    "SELL  {} [{:.0} platinum] [ID {}]",
                    order.item.en.item_name, order.platinum, order.item.id
                ));
            }

            ui.add_space(20.0);

            for order in orders.buy_orders.iter() {
                ui.label(format!(
                    "BUY   {} [{:.0} platinum] [ID {}]",
                    order.item.en.item_name, order.platinum, order.item.id
                ));
            }
        })
    }

    fn should_close(&self, app: &App) -> bool {
        false
    }
}
