use anyhow::bail;
use crossbeam_channel::Sender;
use eframe::egui::{Context, Image, ImageSource, Ui, Color32};
use eframe::egui::RichText;
use eguikit::Spinner;
use eguikit::spinner::Style;
use log::trace;
use tokio::runtime::Runtime;
use wfm_rs::response::{MarketStatisticsWrapper, MarketSubStatisticsClosed, ShortItem};
use wfm_rs::User;
use egui_plot::{BoxElem, BoxPlot, BoxSpread, Legend, Line, Plot};
use crate::app::{App, AppEvent, AppWindow};
use crate::worker::Job;

pub const WFM_STATIC_PREFIX: &'static str = "https://warframe.market/static/assets/";

pub struct ItemDetailsApp {
    pub item: ShortItem,
    pub market_stats: Option<MarketStatisticsWrapper>,
    last_48h_stats: Option<MarketSubStatisticsClosed>,
    median: Vec<[f64; 2]>,
    sma: Vec<[f64; 2]>
}

impl ItemDetailsApp {
    pub fn new(item: ShortItem) -> Self {
        Self {
            item,
            market_stats: None,
            last_48h_stats: None,
            median: Vec::new(),
            sma: Vec::new()
        }
    }

    fn build_boxplot(&mut self) {
        if self.market_stats.is_none() {
            return;
        }

        let mut stats = self.market_stats.as_ref().unwrap().statistics_closed._48_hours.clone();
        stats.sort_by(|a, b| a.datetime.cmp(&b.datetime));
        self.last_48h_stats = Some(stats.last().unwrap().clone());
        for (idx, i) in stats.iter().enumerate() {
            self.median.push([idx as _, i.median as _]);
            self.sma.push([idx as _, i.wa_price as _]);
        };
    }
}

impl AppWindow for ItemDetailsApp {
    fn init(&mut self, app: &App) {
        let user = app.with_user(|u| u.cloned()).unwrap();

        app.submit_job(GetMarketStatisticsJob {
            item: self.item.clone(),
            user
        }).unwrap();
    }

    fn show_close_button(&self) -> bool {
        true
    }

    fn window_title(&self) -> String {
        format!("Details for {}", self.item.item_name)
    }

    fn update(&mut self, app: &App, ctx: &Context, ui: &mut Ui) {
        if self.market_stats.is_none() {
            if app.present_in_storage(&get_market_statistics_storage_key(&self.item)) {
                self.market_stats = app.get_from_storage::<MarketStatisticsWrapper, _, _>(&get_market_statistics_storage_key(&self.item), |v| {
                    Some(v.unwrap().clone())
                });

                self.build_boxplot();
            } else {
                ui.add_space(50.0);
                ui.vertical_centered(|ui| {
                    ui.add(Spinner::default().style(Style::Dots));
                    ui.add_space(5.0);
                    ui.label("Loading market statistics");
                });
                ui.add_space(50.0);
                return;
            }
        }

        ui.horizontal(|ui| {
            ui.add(Image::new(ImageSource::Uri(format!("{}{}", WFM_STATIC_PREFIX, self.item.thumb).into()))
                .show_loading_spinner(true)
                .fit_to_exact_size([128.0, 128.0].into()));

            let statslast48 = self.last_48h_stats.as_ref().unwrap();

            ui.vertical(|ui| {
                display_property(ui, "name", &self.item.item_name);
                display_property(ui, "48h median", &format!("{:.1} platinum", statslast48.median));
                display_property(ui, "48h average", &format!("{:.1} platinum", statslast48.avg_price))
            });
        });

        Plot::new("market-price-plot")
            .legend(Legend::default())
            .allow_zoom(true)
            .allow_drag(true)
            .auto_bounds_x()
            .auto_bounds_y()
            .show(ui, |ui| {
                ui.line(Line::new(self.median.clone()).name("Median price"));
                ui.line(Line::new(self.sma.clone()).name("Moving average price"));
            });
    }
}

fn display_property(ui: &mut Ui, name: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(name).italics().weak());
        ui.add_space(5.0);
        ui.label(value);
    });
}

pub struct GetMarketStatisticsJob {
    item: ShortItem,
    user: User
}

impl Job for GetMarketStatisticsJob {
    fn on_submit(&mut self, app: &App) -> anyhow::Result<()> {
        if app.present_in_storage(&format!("{}_pending", get_market_statistics_storage_key(&self.item))) {
            bail!("Already pending!");
        }

        if app.present_in_storage(&get_market_statistics_storage_key(&self.item)) {
            trace!("Attempt to fetch already loaded market statistics for {}!", self.item.item_name);
            bail!("Market statistics already present in storage!");
        }

        Ok(())
    }

    fn run(&mut self, rt: &Runtime, tx: &Sender<AppEvent>) -> anyhow::Result<()> {
        let stats = rt.block_on(self.user.get_item_market_statistics(&self.item))?;
        tx.send(AppEvent::InsertIntoStorage(
            get_market_statistics_storage_key(&self.item),
            Box::new(stats)
        ))?;
        tx.send(AppEvent::RemoveFromStorage(format!("{}_pending", get_market_statistics_storage_key(&self.item))))?;

        Ok(())
    }
}

fn get_market_statistics_storage_key(item: &ShortItem) -> String {
    format!("market_statistics_{}_{}", item.item_name, item.id)
}
