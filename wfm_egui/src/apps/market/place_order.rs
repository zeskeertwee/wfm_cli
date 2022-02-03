use crate::app::{App, AppWindow};
use crate::background_jobs::item_orders::{FetchItemOrdersJob, ItemOrders};
use crate::util::clone_option_with_inner_ref;
use eframe::egui::{ComboBox, CtxRef, Grid, ScrollArea, Ui};
use std::cmp::max;
use wfm_rs::response::ShortItem;
use wfm_rs::shared::OrderType;

pub struct PlaceOrderPopup {
    item: ShortItem,
    kind: OrderType,
    price_text: String,
}

impl PlaceOrderPopup {
    pub fn new(item: ShortItem, kind: OrderType) -> Self {
        Self {
            item,
            kind,
            price_text: "".to_string(),
        }
    }
}

impl AppWindow for PlaceOrderPopup {
    fn window_title(&self) -> String {
        format!("Placing order for {}", self.item.item_name)
    }

    fn update(&mut self, app: &App, ctx: &CtxRef, ui: &mut Ui) {
        ComboBox::from_label("Select order kind")
            .selected_text(format!("{:?}", self.kind))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.kind, OrderType::Buy, "Buy");
                ui.selectable_value(&mut self.kind, OrderType::Sell, "Sell");
            });

        ui.horizontal(|ui| {
            ui.label("Price: ");
            ui.text_edit_singleline(&mut self.price_text);
            ui.label(" Platinum")
        });

        ui.add_space(20.0);
        if ui.button("Place order").clicked() {
            log::info!(
                "STUB: Placed order: {:?} {} for {}p",
                self.kind,
                self.item.item_name,
                self.price().unwrap_or(0)
            );
        }

        if let Some(orders) = app.get_from_storage::<ItemOrders, _, _>(
            &crate::background_jobs::item_orders::item_orders_storage_key(&self.item),
            |v| clone_option_with_inner_ref(v),
        ) {
            let (mut sell_orders, mut buy_orders) = orders.iter().fold(
                (vec![], vec![]),
                |(mut sell_orders, mut buy_orders), order| {
                    match order.order_type {
                        OrderType::Buy => buy_orders.push(order),
                        OrderType::Sell => sell_orders.push(order),
                    }
                    (sell_orders, buy_orders)
                },
            );

            buy_orders.sort_by(|a, b| a.platinum.partial_cmp(&b.platinum).unwrap());
            buy_orders.reverse();
            sell_orders.sort_by(|a, b| a.platinum.partial_cmp(&b.platinum).unwrap());

            ScrollArea::new([false, true]).show(ui, |ui| {
                Grid::new("existing_orders")
                    .striped(true)
                    .num_columns(2)
                    .show(ui, |ui| {
                        for i in 0..max(sell_orders.len(), buy_orders.len()) {
                            match buy_orders.get(i) {
                                Some(order) => ui.label(format!(
                                    "BUY {}x {} platinum [{}]",
                                    order.quantity, order.platinum, order.user.status
                                )),
                                None => ui.label(""),
                            };

                            match sell_orders.get(i) {
                                Some(order) => ui.label(format!(
                                    "SELL {}x {} platinum [{}]",
                                    order.quantity, order.platinum, order.user.status
                                )),
                                None => ui.label(""),
                            };

                            ui.end_row();
                        }
                    });
            });
        } else {
            ui.label("Loading orders...");
        }
    }

    fn init(&mut self, app: &App) {
        if let Some(user) = app.with_user(|user| clone_option_with_inner_ref(user)) {
            app.submit_job(FetchItemOrdersJob::new(user, self.item.clone()))
                .unwrap();
        }
    }
}

impl PlaceOrderPopup {
    pub fn price(&self) -> anyhow::Result<u64> {
        Ok(self.price_text.parse()?)
    }
}
