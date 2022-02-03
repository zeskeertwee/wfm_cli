use crate::app::{App, AppEvent};
use crate::util::clone_option_with_inner_ref;
use crate::worker::{send_over_tx, Job};
use anyhow::anyhow;
use crossbeam_channel::Sender;
use std::time::Instant;
use tokio::runtime::Runtime;
use wfm_rs::response::{Order, ShortItem};
use wfm_rs::User;

pub struct FetchItemOrdersJob {
    user: User,
    item: ShortItem,
}

pub type ItemOrders = Vec<Order>;
pub const ITEM_ORDERS_EXPIRATION_SECONDS: u64 = 60;

impl Job for FetchItemOrdersJob {
    fn run(&mut self, rt: &Runtime, tx: &Sender<AppEvent>) -> anyhow::Result<()> {
        let orders = rt.block_on(self.user.get_item_orders(&self.item))?;

        send_over_tx(
            tx,
            AppEvent::InsertIntoStorage(self.storage_key(), Box::new(orders)),
        )?;
        send_over_tx(tx, AppEvent::RemoveFromStorage(self.pending_storage_key()))?;
        send_over_tx(
            tx,
            AppEvent::InsertIntoStorage(self.timestamp_storage_key(), Box::new(Instant::now())),
        )?;

        Ok(())
    }

    fn on_submit(&mut self, app: &App) -> anyhow::Result<()> {
        if let Some(timestamp) = app
            .get_from_storage::<Instant, _, _>(&self.timestamp_storage_key(), |t| {
                clone_option_with_inner_ref(t)
            })
        {
            let valid_for = ITEM_ORDERS_EXPIRATION_SECONDS - timestamp.elapsed().as_secs();
            if valid_for > 0 {
                anyhow::bail!(
                    "Item orders already loaded are still valid for {}s!",
                    valid_for
                );
            }
        }

        if app.present_in_storage(&self.pending_storage_key()) {
            anyhow::bail!("Item order fetch already pending");
        }

        app.insert_into_storage(&self.pending_storage_key(), ());

        Ok(())
    }
}

impl FetchItemOrdersJob {
    fn storage_key(&self) -> String {
        item_orders_storage_key(&self.item)
    }

    fn pending_storage_key(&self) -> String {
        format!("{}_pending", self.storage_key())
    }

    fn timestamp_storage_key(&self) -> String {
        format!("{}_timestamp", self.storage_key())
    }

    pub fn new(user: User, item: ShortItem) -> Self {
        Self { user, item }
    }
}

pub fn item_orders_storage_key(item: &ShortItem) -> String {
    format!("item_orders_{}_{}", item.item_name, item.id)
}
