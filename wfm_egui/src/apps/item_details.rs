use anyhow::bail;
use crossbeam_channel::Sender;
use eframe::egui::{Context, Image, ImageSource, Ui};
use eguikit::Spinner;
use eguikit::spinner::Style;
use log::trace;
use tokio::runtime::Runtime;
use wfm_rs::response::{MarketStatisticsWrapper, ShortItem};
use wfm_rs::User;
use crate::app::{App, AppEvent, AppWindow};
use crate::worker::Job;

pub struct ItemDetailsApp {
    pub item: ShortItem,
    pub market_stats: Option<MarketStatisticsWrapper>,
}

impl ItemDetailsApp {
    pub fn new(item: ShortItem) -> Self {
        Self {
            item,
            market_stats: None,
        }
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
        ui.horizontal(|ui| {
            ui.add(Image::new(ImageSource::Uri(self.item.thumb.clone().into()))
                .show_loading_spinner(true));
        });
    }
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