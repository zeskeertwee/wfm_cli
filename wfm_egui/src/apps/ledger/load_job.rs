use anyhow::bail;
use crossbeam_channel::Sender;
use log::warn;
use tokio::runtime::Runtime;
use crate::app::{App, AppEvent};
use crate::apps::ledger::{Ledger, LEDGER_KEY};
use crate::worker::Job;

pub struct LedgerLoadJob;

const LEDGER_PENDING_KEY: &'static str = "__WMI_LEDGER_PENDING";

impl Job for LedgerLoadJob {
    fn on_submit(&mut self, app: &App) -> anyhow::Result<()> {
        if app.present_in_storage(LEDGER_PENDING_KEY) {
            bail!("Already pending!");
        }

        app.insert_into_storage(LEDGER_PENDING_KEY, ());

        Ok(())
    }

    fn run(&mut self, _rt: &Runtime, tx: &Sender<AppEvent>) -> anyhow::Result<()> {
        let ledger = match Ledger::load_from_disk() {
            Ok(v) => v,
            Err(e) => {
                warn!("Failed to load ledger from disk: {}", e);
                Ledger::default()
            }
        };

        tx.send(AppEvent::RemoveFromStorage(LEDGER_PENDING_KEY.to_string()))?;
        tx.send(AppEvent::InsertIntoStorage(LEDGER_KEY.to_string(), Box::new(ledger)))?;
        Ok(())
    }
}