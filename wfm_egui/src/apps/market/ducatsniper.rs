use std::collections::VecDeque;
use std::time::Instant;
use eframe::egui::{Context, Grid, Slider, Ui};
use egui_plot::HLine;
use log::trace;
use wfm_rs::shared::OrderType;
use wfm_rs::websocket::{WebsocketMessagePayload, WebsocketOrder};
use crate::app::{App, AppWindow, WFM_WS_EVENTS_KEY};

pub struct DucatSniperApp {
    cutoff_slider: f64,
    orders: VecDeque<(WebsocketOrder, Instant)>
}

impl Default for DucatSniperApp {
    fn default() -> Self {
        Self {
            cutoff_slider: 10.0,
            orders: VecDeque::new()
        }
    }
}

impl AppWindow for DucatSniperApp {
    fn window_title(&self) -> String {
        "DucatSniper".to_string()
    }

    fn update(&mut self, app: &App, ctx: &Context, ui: &mut Ui) {
        app.get_from_storage::<Vec<WebsocketMessagePayload>, _, _>(WFM_WS_EVENTS_KEY, |v| if let Some(events) = v {
            for msg in events {
                match msg {
                    WebsocketMessagePayload::NewOrder { order } => {
                        if order.order_type == OrderType::Buy {
                            continue;
                        }

                        if let Some(ducats) = order.item.ducats {
                            let ducats_per_plat = ducats as f64 / order.platinum;
                            trace!("{} d/p on {}", ducats_per_plat, order.item.en.item_name);

                            if ducats_per_plat >= self.cutoff_slider {
                                self.orders.push_front((order.clone(), Instant::now()));
                            }
                        }
                    },
                    _ => (),
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Minimum ducats per platinum");
            ui.add(Slider::new(&mut self.cutoff_slider, 0.0..=25.0));
        });

        ui.add_space(10.0);

        Grid::new("ducatsniper-grid")
            .striped(true)
            .num_columns(6)
            .min_col_width(0.0)
            .show(ui, |ui| {
                ui.label("Item");
                ui.label("Qty");
                ui.label("Plat");
                ui.label("Ducats");
                ui.label("Ratio");
                ui.label("Age");
                ui.end_row();

                for (order, inst) in &self.orders {
                    ui.label(&order.item.en.item_name);
                    ui.label(format!("{}x", order.quantity));
                    ui.label(format!("{}p", order.platinum));
                    ui.label(format!("{}d", order.item.ducats.unwrap()));
                    ui.label(format!("{:.1} d/p", order.item.ducats.unwrap() as f64 / order.platinum));
                    ui.label(format!("{}s ago", inst.elapsed().as_secs()));
                    if ui.button("Copy whisper").clicked() {
                        ctx.copy_text(format!("/w {} Hi! I want to buy: \"{}\" for {} platinum. (from Warframe Inventory Manager)", order.user.ingame_name, order.item.en.item_name, order.platinum))
                    }
                    ui.end_row();
                }
            });
    }
}