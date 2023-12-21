use eframe::egui::panel::TopBottomSide;
use eframe::egui::{Color32, ComboBox, Context, TopBottomPanel};

use wfm_rs::response::ExistingProfileOrders;
use wfm_rs::websocket::{Status, WebsocketMessagePayload};

use crate::app::{App, Notification, WFM_NOTIFICATIONS_KEY, WFM_WS_EVENTS_KEY};
use crate::background_jobs::wfm_profile_orders::WFM_EXISTING_PROFILE_ORDERS_KEY;

const WFM_STATUS_KEY: &'static str = "wfm_status";

pub fn draw_top_menu_bar(app: &App, ctx: &Context) {
    let status = app.get_from_storage::<Vec<WebsocketMessagePayload>, _, _>(WFM_WS_EVENTS_KEY, |v| {
        if let Some(v) = v {
            for i in v {
                match i {
                    WebsocketMessagePayload::SetStatus(status) => return Some(status.clone()),
                    _ => (),
                }
            }
        }

        None
    });

    if !app.present_in_storage(WFM_STATUS_KEY) || status.is_some() {
        app.insert_into_storage(WFM_STATUS_KEY,status)
    }

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

            ui.add_space(10.0);
            ui.label(format!("Warframe.market online status: {}", app.get_from_storage::<Option<Status>, _, _>(WFM_STATUS_KEY, |status| match status.unwrap() {
                Some(Status::InGame) => "In-game",
                Some(Status::Online) => "Online",
                Some(Status::Offline) => "Offline",
                None => "[Unknown]"
            })));
            //app.get_from_storage_mut_or_insert_default::<Option<Status>, _, _>("topbar-online-status-selector", |mut s| {
            //    ComboBox::from_label("WFM online status")
            //        .selected_text(format!("{:?}", s))
            //        .show_ui(ui, |ui| {
            //            ui.selectable_value(s, Some(Status::InGame), "Ingame");
            //            ui.selectable_value(s, Some(Status::Online), "Online");
            //            ui.selectable_value(s, Some(Status::Offline), "Offline");
            //        })
            //});

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

                    let wfm_live_view = ui.button("Live view");
                    if wfm_live_view.clicked() {
                        app.queue_window_spawn(
                            crate::apps::market::live_view::LiveMarketApp::default(),
                        );
                    }
                    wfm_live_view.on_hover_text("View live orders as they are placed on warframe.market");

                    let wfm_ducatsniper = ui.button("DucatSniper");
                    if wfm_ducatsniper.clicked() {
                        app.queue_window_spawn(
                            crate::apps::market::ducatsniper::DucatSniperApp::default()
                        );
                    }
                    wfm_ducatsniper.on_hover_text("Watches live orders and shows the ones with a great ducat/plat ratio");
                });
            })
        })
    });

    TopBottomPanel::new(TopBottomSide::Bottom, "bot_menu_bar").show(ctx, |ui| {
        let notification = app.get_from_storage(WFM_NOTIFICATIONS_KEY, |v: Option<&Vec<Notification>>| {
            match v {
                Some(v) => v.last().cloned(),
                None => None
            }
        });

        match notification {
            Some(v) => { ui.horizontal(|ui| {
                ui.colored_label(Color32::LIGHT_BLUE, format!("[{}]", v.source));
                ui.add_space(5.0);
                ui.label(&v.message);
            }); },
            None => { ui.label("No notifications!"); },
        }
    });
}
