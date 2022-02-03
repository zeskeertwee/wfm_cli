use std::any::Any;
use std::sync::atomic::AtomicU64;
use std::time::{Duration, Instant, SystemTime};

use ahash::AHashMap;
use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui::{CtxRef, Id, Rgba, TextureId, Window};
use eframe::epi::{Frame, Storage};
use eframe::{egui, epi};
use log::{info, trace, warn};
use parking_lot::Mutex;

use wfm_rs::User;

use crate::apps::authenticate::WarframeMarketAuthenticationWindow;
use crate::background_jobs::wfm_manifest::{WarframeMarketManifest, WarframeMarketManifestLoadJob};
use crate::background_jobs::wfm_profile_orders::{
    FetchExistingProfileOrdersJob, WFM_EXISTING_PROFILE_ORDERS_EXPIRATION_SECONDS,
    WFM_EXISTING_PROFILE_ORDERS_KEY, WFM_EXISTING_PROFILE_ORDERS_TIMESTAMP_KEY,
};
use crate::texture::TextureSource;
use crate::util::clone_option_with_inner_ref;
use crate::worker::{Job, WorkerPool};

const IMAGE_PLACEHOLDER_PNG: &[u8] = include_bytes!("../placeholder_texture.png");
pub(crate) const WFM_MANIFEST_KEY: &str = "wfm_manifest";
pub(crate) const WFM_MANIFEST_PENDING_KEY: &str = "wfm_manifest_pending";
pub(crate) const WFM_USER_KEY: &str = "wfm_user";
const WFM_MANIFEST_EXPIRATION_SECONDS: u64 = 60 * 60 * 24;

pub trait AppWindow: Send + 'static {
    fn window_title(&self) -> &str;
    fn update(&mut self, app: &App, ctx: &CtxRef, ui: &mut egui::Ui);
    fn should_close(&self, app: &App) -> bool;

    fn show_close_button(&self) -> bool {
        true
    }
}

const JTW_TOKEN_KEY: &str = "wfm_rs_jwt_token";
const USERNAME_KEY: &str = "wfm_rs_username";

pub struct App {
    id_counter: AtomicU64,
    rx: Receiver<AppEvent>,
    tx: Sender<AppEvent>,
    spawn_queue: Mutex<Vec<Box<dyn AppWindow>>>,
    app_windows: Mutex<AHashMap<u64, (Box<dyn AppWindow>, bool)>>,
    worker_pool: Mutex<WorkerPool>,
    placeholder_texture: Option<egui::TextureId>,
    textures: Mutex<AHashMap<String, egui::TextureId>>,
    storage: Mutex<AHashMap<String, Box<dyn Any>>>,
    last_background_tasks_update: Instant,
}

impl Default for App {
    fn default() -> Self {
        let (tx, rx) = unbounded();
        let tx_clone = tx.clone();

        Self {
            id_counter: AtomicU64::new(0),
            rx,
            tx,
            spawn_queue: Mutex::new(Vec::new()),
            app_windows: Mutex::new(AHashMap::new()),
            worker_pool: Mutex::new(WorkerPool::new(tx_clone)),
            placeholder_texture: None,
            textures: Mutex::new(AHashMap::new()),
            storage: Mutex::new(AHashMap::new()),
            last_background_tasks_update: Instant::now(),
        }
    }
}

impl App {
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

    pub fn queue_window_spawn<T: 'static + AppWindow>(&self, window: T) {
        trace!("Queued window spawn: {}", std::any::type_name::<T>());
        self.spawn_queue.lock().push(Box::new(window));
    }

    fn spawn_window(&self, window: Box<dyn AppWindow>) {
        let id = self
            .id_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
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
    }

    pub fn placeholder_image(&self) -> TextureId {
        match self.placeholder_texture {
            Some(texture) => texture,
            None => panic!("Placeholder texture not loaded"),
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

    pub fn remove_from_storage(&self, key: &str) {
        self.storage.lock().remove(key);

        log::trace!("Removed from storage: {}", key);
    }

    pub fn present_in_storage(&self, key: &str) -> bool {
        self.storage.lock().contains_key(key)
    }

    fn run_background_jobs(&self) {
        if !self.present_in_storage(WFM_MANIFEST_PENDING_KEY) {
            let manifest_timestamp = self.get_from_storage::<WarframeMarketManifest, _, _>(
                WFM_MANIFEST_KEY,
                |manifest| {
                    manifest
                        .and_then(|manifest| Some(manifest.timestamp))
                        .unwrap_or(0)
                },
            );

            if (SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - manifest_timestamp)
                >= WFM_MANIFEST_EXPIRATION_SECONDS
            {
                info!("Manifest expired, loading new one");
                if let Some(user) = self.with_user(|user| user.and_then(|user| Some(user.clone())))
                {
                    self.submit_job(WarframeMarketManifestLoadJob::new(user))
                        .unwrap();
                } else {
                    log::warn!("Could not refresh manifest due to missing user");
                }
            }
        }

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
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "WIM - Warframe Inventory Manager"
    }

    fn setup(&mut self, _ctx: &CtxRef, frame: &Frame, storage: Option<&dyn Storage>) {
        info!("Spawning 1 worker thread");
        self.worker_pool.lock().spawn_worker().unwrap();

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

        let placeholder_source =
            pollster::block_on(TextureSource::Data(IMAGE_PLACEHOLDER_PNG.to_owned()).load())
                .unwrap();
        self.placeholder_texture = Some(placeholder_source.allocate(frame).texture_id().to_owned());

        if let Some(manifest) = storage.get_string(WFM_MANIFEST_KEY) {
            match serde_json::from_str::<WarframeMarketManifest>(&manifest) {
                Ok(manifest) => {
                    self.insert_into_storage(WFM_MANIFEST_KEY, manifest);
                    log::info!("Manifest loaded from persistent storage");
                }
                Err(err) => {
                    log::error!("Failed to deserialize manifest: {}", err);
                }
            }
        }

        self.run_background_jobs();

        self.queue_window_spawn(crate::apps::test::TestApp {})
    }

    fn update(&mut self, ctx: &CtxRef, _frame: &Frame) {
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

        self.get_from_storage::<WarframeMarketManifest, _, _>(WFM_MANIFEST_KEY, |manifest| {
            match manifest {
                Some(manifest) => {
                    storage.set_string(WFM_MANIFEST_KEY, serde_json::to_string(manifest).unwrap());
                }
                None => warn!("No manifest to save to storage!"),
            }
        });
    }

    fn clear_color(&self) -> Rgba {
        Rgba::from_rgb(15.0 / 255.0, 15.0 / 255.0, 15.0 / 255.0)
    }
}

pub enum AppEvent {
    QueueWindowSpawn(Box<dyn AppWindow>),
    InsertIntoStorage(String, Box<dyn Any + Send + 'static>),
    RemoveFromStorage(String),
}
