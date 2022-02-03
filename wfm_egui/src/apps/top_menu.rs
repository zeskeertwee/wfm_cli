use eframe::egui::panel::TopBottomSide;
use eframe::egui::{CtxRef, TopBottomPanel};

use wfm_rs::response::ExistingProfileOrders;

use crate::app::App;
use crate::background_jobs::wfm_profile_orders::WFM_EXISTING_PROFILE_ORDERS_KEY;

pub fn draw_top_menu_bar(app: &App, ctx: &CtxRef) {
    let order_text = app.get_from_storage::<ExistingProfileOrders, _, _>(
        WFM_EXISTING_PROFILE_ORDERS_KEY,
        |orders| match orders {
            Some(orders) => format!(
                ", loaded {}s/{}b orders",
                orders.sell_orders.len(),
                orders.buy_orders.len()
            ),
            None => "".to_string(),
        },
    );

    TopBottomPanel::new(TopBottomSide::Top, "top_menu_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(app.with_user(|user| match user {
                Some(user) => {
                    format!("Signed in as {}{}", user.username(), order_text)
                }
                None => "Not signed in to warframe.market".to_string(),
            }));

            ui.menu_button("Applications", |ui| {
                ui.menu_button("Warframe.market", |ui| {
                    let wfm_item_search = ui.button("Item Search");
                    if wfm_item_search.clicked() {
                        app.queue_window_spawn(
                            crate::apps::market::item_search::ItemSearchApp::default(),
                        );
                    }
                    wfm_item_search.on_hover_text("Search for items on warframe.market");

                    let wfm_item_search = ui.button("Orders");
                    if wfm_item_search.clicked() {
                        app.queue_window_spawn(
                            crate::apps::market::orders::ExistingProfileOrdersApp::default(),
                        );
                    }
                    wfm_item_search.on_hover_text("Open orders on warframe.market");
                });
            })
        })
    });
}
