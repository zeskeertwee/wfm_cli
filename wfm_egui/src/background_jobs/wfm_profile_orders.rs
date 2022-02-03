use crate::app::{App, AppEvent};
use crate::worker::{send_over_tx, Job};
use crossbeam_channel::Sender;
use std::time::SystemTime;
use tokio::runtime::Runtime;
use wfm_rs::User;

pub const WFM_EXISTING_PROFILE_ORDERS_KEY: &str = "existing_profile_orders";
pub const WFM_EXISTING_PROFILE_ORDERS_PENDING_KEY: &str = "existing_profile_orders_pending";
pub const WFM_EXISTING_PROFILE_ORDERS_TIMESTAMP_KEY: &str = "existing_profile_orders_timestamp";
pub const WFM_EXISTING_PROFILE_ORDERS_EXPIRATION_SECONDS: u64 = 120;

pub struct FetchExistingProfileOrdersJob {
    user: User,
}

impl FetchExistingProfileOrdersJob {
    pub fn new(user: User) -> Self {
        Self { user }
    }
}

impl Job for FetchExistingProfileOrdersJob {
    fn run(&mut self, rt: &Runtime, tx: &Sender<AppEvent>) -> anyhow::Result<()> {
        // TODO: Remove this when done testing
        std::thread::sleep_ms(2000);
        let orders = rt.block_on(self.user.get_user_orders())?;

        send_over_tx(
            tx,
            AppEvent::RemoveFromStorage(WFM_EXISTING_PROFILE_ORDERS_PENDING_KEY.to_string()),
        )?;

        send_over_tx(
            tx,
            AppEvent::InsertIntoStorage(
                WFM_EXISTING_PROFILE_ORDERS_KEY.to_string(),
                Box::new(orders),
            ),
        );

        send_over_tx(
            tx,
            AppEvent::InsertIntoStorage(
                WFM_EXISTING_PROFILE_ORDERS_TIMESTAMP_KEY.to_string(),
                Box::new(SystemTime::now()),
            ),
        )?;

        Ok(())
    }

    fn on_submit(&mut self, app: &App) -> anyhow::Result<()> {
        if app.present_in_storage(WFM_EXISTING_PROFILE_ORDERS_PENDING_KEY) {
            return Err(anyhow::bail!("Already pending"));
        }

        app.insert_into_storage(WFM_EXISTING_PROFILE_ORDERS_PENDING_KEY, ());
        Ok(())
    }
}
