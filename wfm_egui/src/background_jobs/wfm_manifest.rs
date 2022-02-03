use crate::app::{App, AppEvent};
use crate::apps::spinner_popup::SpinnerPopup;
use crate::worker::{send_over_tx, Job};
use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime;
use wfm_rs::User;

#[derive(Serialize, Deserialize)]
pub struct WarframeMarketManifest {
    pub timestamp: u64,
    pub items: Arc<Vec<wfm_rs::response::ShortItem>>,
}

impl Clone for WarframeMarketManifest {
    fn clone(&self) -> Self {
        Self {
            timestamp: self.timestamp.clone(),
            items: Arc::clone(&self.items),
        }
    }
}

pub struct WarframeMarketManifestLoadJob {
    user: User,
}

impl WarframeMarketManifestLoadJob {
    pub fn new(user: User) -> Self {
        Self { user }
    }
}

impl Job for WarframeMarketManifestLoadJob {
    fn run(&mut self, rt: &Runtime, tx: &Sender<AppEvent>) -> anyhow::Result<()> {
        std::thread::sleep_ms(5000);

        let items = rt.block_on(self.user.get_items())?;
        let manifest = WarframeMarketManifest {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            items: Arc::new(items),
        };

        send_over_tx(
            tx,
            AppEvent::InsertIntoStorage(
                crate::app::WFM_MANIFEST_KEY.to_string(),
                Box::new(manifest),
            ),
        )?;

        send_over_tx(
            tx,
            AppEvent::RemoveFromStorage(crate::app::WFM_MANIFEST_PENDING_KEY.to_string()),
        );

        Ok(())
    }

    fn on_submit(&mut self, app: &App) -> anyhow::Result<()> {
        if app.present_in_storage(crate::app::WFM_MANIFEST_PENDING_KEY) {
            anyhow::bail!("Manifest load job is already pending");
        }

        app.insert_into_storage(crate::app::WFM_MANIFEST_PENDING_KEY, ());

        SpinnerPopup::new(
            &app,
            "Loading",
            "Updating warframe.market item manifest...",
            Some("Finished updating warframe.market item manifest"),
            crate::app::WFM_MANIFEST_PENDING_KEY,
            5.0,
        );

        Ok(())
    }
}
