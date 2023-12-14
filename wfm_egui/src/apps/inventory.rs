use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use eframe::egui::{Color32, CtxRef, Grid, Ui};
use eframe::epi::Storage;
use log::trace;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use wfm_rs::response::ShortItem;
use serde_json;
use serde_with::serde_as;
use wfm_rs::shared::OrderType;
use crate::app::{App, AppWindow};

pub struct InventoryApp {
    pub search_field: String,
}

impl AppWindow for InventoryApp {
    fn window_title(&self) -> String {
        "Inventory".to_string()
    }

    fn update(&mut self, app: &App, ctx: &CtxRef, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Search");
            ui.add_space(5.0);
            ui.text_edit_singleline(&mut self.search_field);
        });

        app.get_from_storage::<Inventory, _, _>(INVENTORY_KEY, |i| {
            let mut cleanup_needed = false;
            let i = i.unwrap();
            Grid::new("inventory_items")
                .striped(true)
                .num_columns(5)
                .min_col_width(0.0)
                .show(ui, |ui| {
                    for (item, amount) in i.items.lock().iter_mut().filter(|item| item.0.item_name.to_lowercase().contains(&self.search_field)) {
                        ui.colored_label(Color32::LIGHT_GREEN, format!("{}x", amount));
                        ui.label(&item.item_name);
                        if ui.button("-1").clicked() {
                            *amount -= 1;
                        }
                        if ui.button("+1").clicked() {
                            *amount += 1;
                        }

                        if *amount == 0 {
                            cleanup_needed = true;
                        }

                        if ui.button("Details").clicked() {
                            app.queue_window_spawn(super::item_details::ItemDetailsApp::new(item.clone()));
                        }

                        ui.end_row();
                    }
                });

            if cleanup_needed {
                i.cleanup();
            }
        })
    }
}

pub const INVENTORY_KEY: &str = "__wmi_inventory";

pub struct Inventory {
    items: Arc<Mutex<InnerInventory>>,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
struct InnerInventory {
    #[serde_as(as = "Vec<(_, _)>")]
    inner: HashMap<ShortItem, u16>
}

impl Deref for InnerInventory {
    type Target = HashMap<ShortItem, u16>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for InnerInventory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Clone for Inventory {
    fn clone(&self) -> Self {
        Self {
            items: Arc::clone(&self.items)
        }
    }
}

impl Inventory {
    pub fn insert_item(&self, item: ShortItem, amount: u16) {
        let mut lock = self.items.lock();
        match lock.get_mut(&item) {
            Some(item) => *item += amount,
            None => { lock.insert(item, amount); },
        }
    }

    pub fn load_from_storage(storage: &dyn Storage) -> Self {
        match storage.get_string(INVENTORY_KEY) {
            Some(v) => {
                trace!("Loading inventory from persistent storage");
                Self {
                    items: Arc::new(Mutex::new(serde_json::from_str::<InnerInventory>(&v).unwrap()))
                }
            },
            None => {
                trace!("No inventory in persistent storage, create empty inventory");
                Inventory {
                    items: Arc::new(Mutex::new(InnerInventory { inner: HashMap::new() })),
                }
            }
        }
    }

    pub fn save_to_storage(&self, storage: &mut dyn Storage) {
        if self.items.lock().is_empty() {
            trace!("Not saving empty inventory to storage!");
            return;
        }

        trace!("Saving inventory to persistant storage");
        storage.set_string(INVENTORY_KEY, serde_json::to_string::<InnerInventory>(&self.items.lock()).unwrap());
    }

    /// removes all items with 0 amount
    pub fn cleanup(&self) {
        let to_remove: Vec<ShortItem> = self.items.lock().iter().filter(|v| *v.1 == 0).map(|v| v.0.clone()).collect();

        for i in to_remove {
            self.items.lock().remove(&i);
        }
    }
}