use crate::app::{App, AppEvent, AppWindow, WFM_MANIFEST_KEY};
use crate::background_jobs::wfm_manifest::WarframeMarketManifest;
use crate::background_jobs::wfm_profile_orders::WFM_EXISTING_PROFILE_ORDERS_KEY;
use crate::worker::Job;
use anyhow::bail;
use atomic_float::AtomicF64;
use crossbeam_channel::Sender;
use eframe::egui::panel::TopBottomSide;
use eframe::egui::{CtxRef, Grid, Rgba, RichText, TextEdit, TopBottomPanel, Ui};
use levenshtein::levenshtein;
use parking_lot::Mutex;
use pretty_env_logger::env_logger::fmt::Color;
use std::cmp::min;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Runtime;
use wfm_rs::response::ExistingProfileOrders;
use wfm_rs::response::ShortItem;

const ITEM_SEARCH_PENDING_KEY: &str = "wfm_item_search_pending";

#[derive(Default)]
pub struct ItemSearchApp {
    search_text: String,
    manifest: Option<WarframeMarketManifest>,
    closest_items: Arc<Mutex<Vec<ShortItem>>>,
    search_duration_ms: Arc<AtomicF64>,
}

impl AppWindow for ItemSearchApp {
    fn window_title(&self) -> &str {
        "Warframe.market item search"
    }

    fn update(&mut self, app: &App, ctx: &CtxRef, ui: &mut Ui) {
        if self.manifest.is_none() {
            match app
                .get_from_storage::<WarframeMarketManifest, _, _>(WFM_MANIFEST_KEY, |manifest| {
                    manifest.and_then(|manifest| Some(manifest.clone()))
                }) {
                Some(manifest) => {
                    self.manifest = Some(manifest);
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

        let manifest = self.manifest.as_ref().unwrap();

        let search_field = TextEdit::singleline(&mut self.search_text)
            .hint_text("Search items")
            .show(ui);

        if search_field.response.changed() {
            app.submit_job(FindClosestItemsJob {
                search_text: self.search_text.clone(),
                closest_items: Arc::clone(&self.closest_items),
                manifest: manifest.clone(),
                search_duration_ms: Arc::clone(&self.search_duration_ms),
            })
            .unwrap();
        }

        self.display_closest_items(app, ui);
        TopBottomPanel::new(TopBottomSide::Bottom, "wfm_item_search_bottom_panel").show_inside(
            ui,
            |ui| {
                ui.label(format!(
                    "Search took: {:.2}ms",
                    self.search_duration_ms.load(Ordering::Relaxed)
                ));
            },
        );
    }

    fn should_close(&self, app: &App) -> bool {
        false
    }
}

impl ItemSearchApp {
    fn display_closest_items(&self, app: &App, ui: &mut Ui) {
        let existing_orders = app
            .get_from_storage::<ExistingProfileOrders, _, ExistingProfileOrders>(
                WFM_EXISTING_PROFILE_ORDERS_KEY,
                |orders| match orders {
                    Some(orders) => orders.clone(),
                    None => ExistingProfileOrders {
                        sell_orders: vec![],
                        buy_orders: vec![],
                    },
                },
            );

        Grid::new("closest_item_grid")
            .striped(true)
            .num_columns(3)
            .show(ui, |ui| {
                for item in self.closest_items.lock().iter() {
                    ui.label(&item.item_name);

                    let sell_text = if existing_orders
                        .sell_orders
                        .iter()
                        .filter(|v| v.item.id == item.id)
                        .count()
                        >= 1
                    {
                        "SELL"
                    } else {
                        "    "
                    };
                    ui.label(RichText::new(sell_text).monospace().color(Rgba::from_rgb(
                        27.0 / 255.0,
                        177.0 / 255.0,
                        148.0 / 255.0,
                    )));

                    let buy_text = if existing_orders
                        .buy_orders
                        .iter()
                        .filter(|v| v.item.id == item.id)
                        .count()
                        >= 1
                    {
                        "BUY"
                    } else {
                        "   "
                    };
                    ui.label(RichText::new(buy_text).monospace().color(Rgba::from_rgb(
                        60.0 / 255.0,
                        135.0 / 255.0,
                        156.0 / 255.0,
                    )));

                    ui.end_row();
                }
            });
    }
}

pub struct FindClosestItemsJob {
    search_text: String,
    manifest: WarframeMarketManifest,
    closest_items: Arc<Mutex<Vec<ShortItem>>>,
    search_duration_ms: Arc<AtomicF64>,
}

const KEYWORD_BIAS: f64 = 0.5;
const NON_KEYWORD_BIAS: f64 = 1.2;
const KEYWORDS: [&'static str; 14] = [
    // warframes
    "neuroptics",
    "systems",
    "chassis",
    "blueprint",
    "set",
    // mods
    "primed",
    "galvanized",
    // relics
    "lith",
    "meso",
    "neo",
    "axi",
    "requiem",
    // trivia
    "arcane",
    "prime",
];

impl Job for FindClosestItemsJob {
    fn run(&mut self, rt: &Runtime, tx: &Sender<AppEvent>) -> anyhow::Result<()> {
        let start = Instant::now();
        let search_text_keywords = get_keywords_for_string(&self.search_text);

        // lehvenstein distance, item idx
        let mut distances: Vec<(i64, usize)> = Vec::with_capacity(self.manifest.items.len());

        for (idx, item) in self.manifest.items.iter().enumerate() {
            let distance = levenshtein(&self.search_text, &item.item_name);
            let keywords = get_keywords_for_string(&item.item_name);
            let keyword_score = get_keyword_score(&search_text_keywords, &keywords);

            distances.push((
                (distance as f64 - (distance as f64 * keyword_score)) as i64,
                idx,
            ));
        }

        distances.sort_unstable();

        let mut closest_items = Vec::with_capacity(15);
        for i in 0..15 {
            closest_items.push(self.manifest.items[distances[i].1].clone());
        }

        *self.closest_items.lock() = closest_items;
        self.search_duration_ms.store(
            start.elapsed().as_secs_f64() as f64 * 1000.0,
            Ordering::Relaxed,
        );

        match tx.send(AppEvent::RemoveFromStorage(
            ITEM_SEARCH_PENDING_KEY.to_string(),
        )) {
            Ok(_) => (),
            Err(e) => bail!("{}", e),
        }

        Ok(())
    }

    fn on_submit(&mut self, app: &App) -> anyhow::Result<()> {
        if app.present_in_storage(ITEM_SEARCH_PENDING_KEY) {
            anyhow::bail!("Item search already pending!");
        }

        app.insert_into_storage(ITEM_SEARCH_PENDING_KEY, ());

        Ok(())
    }
}

fn get_keywords_for_string(s: &str) -> Vec<String> {
    s.split_whitespace().map(|s| s.to_lowercase()).collect()
}

/// this score is in a range 0-1
fn get_keyword_score(a: &Vec<String>, b: &Vec<String>) -> f64 {
    if a.len() == 0 || b.len() == 0 {
        return 0.0;
    }

    let mut score: f64 = 0.0;

    for i in a.iter() {
        if b.contains(i) {
            score += if KEYWORDS.contains(&i.as_str()) {
                KEYWORD_BIAS
            } else {
                NON_KEYWORD_BIAS
            };
        }
    }

    score = score / min(a.len(), b.len()) as f64;

    score
}
