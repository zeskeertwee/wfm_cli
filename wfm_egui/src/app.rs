use std::any::Any;
use std::sync::atomic::AtomicU64;
use std::time::{Duration, Instant, SystemTime};

use ahash::AHashMap;
use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui::{Context, Id, Rgba, TextureId, Visuals, Window};
use eframe::{Frame, Storage};
use eframe::egui;
use log::{error, info, trace, warn};
use parking_lot::Mutex;

use wfm_rs::User;
use wfm_rs::websocket::WebsocketMessagePayload;

use crate::apps::authenticate::WarframeMarketAuthenticationWindow;
use crate::apps::inventory::{Inventory, INVENTORY_KEY};
use crate::apps::ledger::{Ledger, LEDGER_KEY};
use crate::background_jobs::wfm_manifest::{WarframeMarketManifest, WarframeMarketManifestLoadJob};
use crate::background_jobs::wfm_profile_orders::{
    FetchExistingProfileOrdersJob, WFM_EXISTING_PROFILE_ORDERS_EXPIRATION_SECONDS,
    WFM_EXISTING_PROFILE_ORDERS_KEY, WFM_EXISTING_PROFILE_ORDERS_TIMESTAMP_KEY,
};
use crate::util::clone_option_with_inner_ref;
use crate::worker::{Job, WorkerPool};

const IMAGE_PLACEHOLDER_PNG: &[u8] = include_bytes!("../placeholder_texture.png");
pub(crate) const WFM_MANIFEST_KEY: &str = "wfm_manifest";
pub(crate) const WFM_MANIFEST_PENDING_KEY: &str = "wfm_manifest_pending";
pub(crate) const WFM_USER_KEY: &str = "wfm_user";
pub(crate) const WFM_NOTIFICATIONS_KEY: &str = "__wfm_notifications";
pub(crate) const WFM_WS_EVENTS_KEY: &str = "__wfm_websocket_events";
const WFM_MANIFEST_EXPIRATION_SECONDS: u64 = 60 * 60 * 24;

pub trait AppWindow: Send + 'static {
    fn window_title(&self) -> String;
    fn update(&mut self, app: &App, ctx: &Context, ui: &mut egui::Ui);

    fn init(&mut self, app: &App) {}
    fn should_close(&self, app: &App) -> bool {
        false
    }
    fn show_close_button(&self) -> bool {
        true
    }
}

const JTW_TOKEN_KEY: &str = "wfm_rs_jwt_token";
const USERNAME_KEY: &str = "wfm_rs_username";

#[derive(Clone)]
pub struct Notification {
    pub timestamp: Instant,
    pub source: String,
    pub message: String,
}

impl Notification {
    pub fn new<T: ToString, U: ToString>(source: T, message: U) -> Notification {
        Self {
            timestamp: Instant::now(),
            source: source.to_string(),
            message: message.to_string()
        }
    }
}

pub struct App {
    id_counter: AtomicU64,
    ws_rx: Receiver<WebsocketMessagePayload>,
    ws_tx: Sender<WebsocketMessagePayload>,
    rx: Receiver<AppEvent>,
    tx: Sender<AppEvent>,
    spawn_queue: Mutex<Vec<Box<dyn AppWindow>>>,
    app_windows: Mutex<AHashMap<u64, (Box<dyn AppWindow>, bool)>>,
    worker_pool: Mutex<WorkerPool>,
    textures: Mutex<AHashMap<String, egui::TextureId>>,
    storage: Mutex<AHashMap<String, Box<dyn Any>>>,
    last_background_tasks_update: Instant,
}

impl Default for App {
    fn default() -> Self {
        let (tx, rx) = unbounded();
        let (ws_tx, ws_rx) = unbounded();
        let tx_clone = tx.clone();

        Self {
            id_counter: AtomicU64::new(0),
            rx, tx,
            ws_rx, ws_tx,
            spawn_queue: Mutex::new(Vec::new()),
            app_windows: Mutex::new(AHashMap::new()),
            worker_pool: Mutex::new(WorkerPool::new(tx_clone)),
            textures: Mutex::new(AHashMap::new()),
            storage: Mutex::new(AHashMap::new()),
            last_background_tasks_update: Instant::now(),
        }
    }
}

impl App {
    pub fn submit_notification(&self, notification: Notification) {
        if !self.present_in_storage(WFM_NOTIFICATIONS_KEY) {
            self.insert_into_storage(WFM_NOTIFICATIONS_KEY, vec![notification]);
            info!("Initialized notification storage");
            return;
        }

        self.get_from_storage_mut(WFM_NOTIFICATIONS_KEY, |v: Option<&mut Vec<Notification>>| {
            match v {
                Some(mut v) => v.push(notification),
                None => error!("Notification storage not initialized!"),
            }
        })
    }

    pub fn submit_job<T: Job>(&self, mut job: T) -> anyhow::Result<()> {
        let name = job.job_name();
        match job.on_submit(&self) {
            Ok(()) => (),
            Err(e) => {
                log::trace!("{} on-submit failed: {}", name, e);
                return Ok(());
            }
        }
        self.worker_pool.lock().sumbit_job(Box::new(job))
    }

    pub fn with_user<F: FnOnce(Option<&User>) -> R, R>(&self, func: F) -> R {
        self.get_from_storage::<User, _, _>(WFM_USER_KEY, |usr| match usr {
            Some(user) => func(Some(user)),
            None => func(None),
        })
    }

    pub fn get_user(&self) -> Option<User> {
        self.with_user(|v| v.map(|u| u.clone()))
    }

    pub fn queue_window_spawn<T: 'static + AppWindow>(&self, window: T) {
        trace!("Queued window spawn: {}", std::any::type_name::<T>());
        self.spawn_queue.lock().push(Box::new(window));
    }

    fn spawn_window(&self, mut window: Box<dyn AppWindow>) {
        let id = self
            .id_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        window.init(&self);
        self.app_windows.lock().insert(id, (window, true));
        trace!("Spawned window with id {}", id);
    }

    fn process_events(&self) {
        loop {
            match self.rx.try_recv() {
                Ok(event) => match event {
                    AppEvent::QueueWindowSpawn(window) => {
                        self.spawn_queue.lock().push(window);
                    }
                    AppEvent::InsertIntoStorage(key, value) => {
                        log::trace!("Inserted into storage: {} (from AppEvent)", key);
                        self.storage.lock().insert(key, value);
                    }
                    AppEvent::RemoveFromStorage(key) => {
                        log::trace!("Removed from storage: {} (from AppEvent)", key);
                        self.storage.lock().remove(&key);
                    }
                },
                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    panic!("app event channel disconnected");
                }
            }
        }

        if self.present_in_storage(WFM_WS_EVENTS_KEY) {
            self.remove_from_storage(WFM_WS_EVENTS_KEY);
        }

        let mut ws_events = Vec::new();
        loop {
            match self.ws_rx.try_recv() {
                Ok(v) => {
                    ws_events.push(v);
                },
                Err(_) => break,
            }
        }

        if !ws_events.is_empty() {
            self.insert_into_storage(WFM_WS_EVENTS_KEY, ws_events);
        }
    }

    pub fn insert_into_storage<T: 'static>(&self, key: &str, value: T) {
        self.storage.lock().insert(key.to_owned(), Box::new(value));

        log::trace!(
            "Inserted into storage: {} ({})",
            key,
            std::any::type_name::<T>()
        );
    }

    pub fn get_from_storage<T: 'static, F: FnOnce(Option<&T>) -> R, R>(
        &self,
        key: &str,
        func: F,
    ) -> R {
        match self.storage.lock().get(key) {
            Some(value) => func(value.downcast_ref::<T>()),
            None => func(None),
        }
    }

    pub fn get_from_storage_mut<T: 'static, F: FnOnce(Option<&mut T>) -> R, R>(
        &self,
        key: &str,
        func: F,
    ) -> R {
        match self.storage.lock().get_mut(key) {
            Some(mut value) => func(value.downcast_mut::<T>()),
            None => func(None),
        }
    }

    pub fn get_from_storage_mut_or_insert_default<T: 'static + Default, F: FnOnce(&mut T) -> R, R>(
        &self,
        key: &str,
        func: F,
    ) -> R {
        match self.storage.lock().get_mut(key) {
            Some(mut value) => match value.downcast_mut::<T>() {
                Some(v) => return func(v),
                None => (),
            },
            None => (),
        }

        self.insert_into_storage(key, T::default());
        match self.storage.lock().get_mut(key) {
            Some(mut value) => return func(value.downcast_mut::<T>().unwrap()),
            None => panic!("Inserted item not present in storage!"),
        }
    }

    pub fn remove_from_storage(&self, key: &str) {
        self.storage.lock().remove(key);

        log::trace!("Removed from storage: {}", key);
    }

    pub fn present_in_storage(&self, key: &str) -> bool {
        self.storage.lock().contains_key(key)
    }

    fn run_background_jobs(&self) {
        if !self.present_in_storage(WFM_EXISTING_PROFILE_ORDERS_KEY) {
            if let Some(user) = self.with_user(|user| user.and_then(|user| Some(user.clone()))) {
                self.submit_job(FetchExistingProfileOrdersJob::new(user))
                    .unwrap();
            } else {
                log::warn!("Could not load profile orders due to missing user");
            }
        } else if let Some(timestamp) = self
            .get_from_storage::<SystemTime, _, _>(WFM_EXISTING_PROFILE_ORDERS_TIMESTAMP_KEY, |t| {
                clone_option_with_inner_ref(t)
            })
        {
            if SystemTime::now()
                .duration_since(timestamp)
                .unwrap()
                .as_secs()
                > WFM_EXISTING_PROFILE_ORDERS_EXPIRATION_SECONDS
            {
                if let Some(user) = self.with_user(|user| user.and_then(|user| Some(user.clone())))
                {
                    self.submit_job(FetchExistingProfileOrdersJob::new(user))
                        .unwrap();
                } else {
                    log::warn!("Could not refresh profile orders due to missing user");
                }
            }
        }

        if let Some(user) = self.get_user() && !self.present_in_storage(WFM_MANIFEST_KEY) {
            self.submit_job(WarframeMarketManifestLoadJob::new(user)).unwrap();
        }
    }

    fn setup(&mut self, ctx: &Context, storage: Option<&dyn Storage>) {
        egui_extras::install_image_loaders(ctx);

	    info!("Spawning 4 worker threads");
        for _ in 0..4 {
            self.worker_pool.lock().spawn_worker().unwrap();
        }

        let storage = storage.unwrap();

        let jwt_token = storage.get_string(JTW_TOKEN_KEY);
        let username = storage.get_string(USERNAME_KEY);

        if let (Some(token), Some(username)) = (jwt_token, username) {
            info!("User {} loaded from persistent storage", username);
            self.insert_into_storage(WFM_USER_KEY, User::_from_jwt_token(&token, &username));
        } else {
            info!("No user found in persistent storage");
            self.queue_window_spawn(WarframeMarketAuthenticationWindow::default());
        }

        if !self.present_in_storage(INVENTORY_KEY) {
            self.insert_into_storage(INVENTORY_KEY, Inventory::load_from_storage(storage));
        }

        self.submit_job(crate::apps::ledger::load_job::LedgerLoadJob).unwrap();

        self.run_background_jobs();

        self.queue_window_spawn(crate::apps::test::TestApp {});
        self.queue_window_spawn(crate::apps::inventory::InventoryApp {search_field: String::new()});
        self.submit_notification(Notification::new("WIM", "Application initialized"));

        let ws_tx = self.ws_tx.clone();
        self.with_user(|v| crate::background_jobs::websocket_listener::start(v.unwrap().to_owned(), ws_tx, ctx.clone()));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if ctx.frame_nr() == 0 {
            self.setup(ctx, frame.storage());
        }

        self.process_events();
        for window in self.spawn_queue.lock().drain(..) {
            self.spawn_window(window);
        }

        let mut to_remove = Vec::new();

        crate::apps::top_menu::draw_top_menu_bar(&self, ctx);

        for (id, (window, is_window_open)) in self.app_windows.lock().iter_mut() {
            if window.should_close(&self) {
                to_remove.push(id.clone());
                continue;
            }

            Window::new(window.window_title())
                .id(Id::new(format!("{}-{}", window.window_title(), id)))
                .open(is_window_open)
                .show(ctx, |ui| {
                    window.update(self, ctx, ui);
                });

            if !*is_window_open {
                to_remove.push(id.clone());
                continue;
            }
        }

        for id in to_remove {
            self.app_windows.lock().remove(&id);
            trace!("Despawned window with id {}", id);
        }

        if self.last_background_tasks_update.elapsed() > Duration::from_secs(1) {
            self.run_background_jobs();
            self.last_background_tasks_update = Instant::now();
        }
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        trace!("Saving app state to persistent storage");
        self.with_user(|user| match user {
            Some(user) => {
                storage.set_string(JTW_TOKEN_KEY, user._jwt_token());
                storage.set_string(USERNAME_KEY, user.username());
            }
            None => warn!("No user to save to storage!"),
        });

        self.get_from_storage::<Inventory, _, _>(INVENTORY_KEY, |i| {
            match i {
                Some(i) => i.save_to_storage(storage),
                None => warn!("No inventory to save to storage!"),
            }
        });

        self.get_from_storage::<Ledger, _, _>(LEDGER_KEY, |l| {
            match l {
                Some(ledger) => ledger.save_to_disk().unwrap(),
                None => warn!("No ledger to save to disk!"),
            }
        });
    }

    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        [15.0 / 255.0, 15.0 / 255.0, 15.0 / 255.0, 1.0]
    }
}

pub enum AppEvent {
    QueueWindowSpawn(Box<dyn AppWindow + Sync>),
    InsertIntoStorage(String, Box<dyn Any + Send + Sync + 'static>),
    RemoveFromStorage(String),
}
