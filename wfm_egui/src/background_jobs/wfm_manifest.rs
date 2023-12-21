use std::io::{Read, Write};
use std::sync::Arc;
use crossbeam_channel::Sender;
use log::info;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;
use wfm_rs::User;
use crate::app::{App, AppEvent};
use crate::apps::spinner_popup::SpinnerPopup;
use crate::util::{create_storage_file, get_storage_file, get_unix_timestamp};
use crate::worker::{send_over_tx, Job};

const WFM_MANIFEST_MAX_AGE: u64 = 24 * 60 * 60; // 1 day

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

impl WarframeMarketManifest {
    fn save_to_disk(&self) {
        let str = serde_json::to_string(self).unwrap();
        let mut file = create_storage_file("wfm_manifest.json").unwrap();
        file.write_all(str.as_bytes()).unwrap();
    }

    fn read_from_disk() -> Option<Self> {
        match get_storage_file("wfm_manifest.json") {
            Ok(mut f) => {
                let mut str = String::new();
                f.read_to_string(&mut str).unwrap();
                serde_json::from_str(&str).unwrap()
            },
            Err(_) => None,
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
        let manifest = match WarframeMarketManifest::read_from_disk() {
            Some(manifest) if ((get_unix_timestamp() - manifest.timestamp) < WFM_MANIFEST_MAX_AGE) => {
                info!("Manifest loaded from disk");
                manifest
            },
            _ => {
                info!("Downloading manifest");
                let items = rt.block_on(self.user.get_items())?;
                let manifest = WarframeMarketManifest {
                    timestamp: get_unix_timestamp(),
                    items: Arc::new(items),
                };
                manifest.save_to_disk();
                manifest
            }
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
        )?;

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
