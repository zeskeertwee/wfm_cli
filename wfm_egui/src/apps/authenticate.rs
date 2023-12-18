use std::sync::Arc;

use crossbeam_channel::Sender;
use eframe::egui::{ComboBox, Context, TextEdit, Ui};
use eguikit::spinner::Style;
use eguikit::Spinner;
use log::info;
use parking_lot::Mutex;
use tokio::runtime::Runtime;

use wfm_rs::User;

use crate::app::{App, AppEvent, AppWindow};
use crate::worker::{Job, JobState};

type AuthenticateState = JobState<User>;

pub struct WarframeMarketAuthenticationWindow {
    email: String,
    password: String,
    platform: wfm_rs::Platform,
    auth_state: Arc<Mutex<AuthenticateState>>,
    error: Option<String>,
    auth_success: bool,
}

impl Default for WarframeMarketAuthenticationWindow {
    fn default() -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            platform: wfm_rs::Platform::Pc,
            auth_state: Arc::new(Mutex::new(AuthenticateState::Idle)),
            error: None,
            auth_success: false,
        }
    }
}

impl AppWindow for WarframeMarketAuthenticationWindow {
    fn window_title(&self) -> String {
        "Warframe.market authentication".to_string()
    }

    fn update(&mut self, app: &App, _ctx: &Context, ui: &mut Ui) {
        if self.auth_success {
            return;
        }

        match &*self.auth_state.lock() {
            AuthenticateState::Idle => (),
            AuthenticateState::Done(r) => match r {
                Ok(user) => {
                    info!("Authentication successful");
                    app.insert_into_storage(crate::app::WFM_USER_KEY, user.clone());
                    self.auth_success = true;
                }
                Err(e) => {
                    self.auth_success = false;
                    self.error = Some(format!("Authentication error: {}", e));
                }
            },
            AuthenticateState::Pending => {
                ui.horizontal(|ui| {
                    ui.add(Spinner::default().style(Style::Dots));
                    ui.add_space(15.0);
                    ui.label("Authenticating...");
                });

                return;
            }
        }

        ui.label("This application will not store your password, it will only be used to authenticate you to the Warframe.market website.");
        ui.add_space(15.0);
        ui.horizontal(|ui| {
            ui.label("Email:");
            ui.text_edit_singleline(&mut self.email);
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("Password:");
            ui.add(TextEdit::singleline(&mut self.password).password(true));
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("Platform:");
            ComboBox::from_label("")
                .selected_text(format!("{:?}", self.platform))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.platform, wfm_rs::Platform::Pc, "Pc");
                    ui.selectable_value(&mut self.platform, wfm_rs::Platform::Ps4, "Ps4");
                    ui.selectable_value(&mut self.platform, wfm_rs::Platform::Xbox, "Xbox");
                });
        });
        ui.add_space(15.0);
        if ui.button("Authenticate").clicked() {
            let job = AuthenticateJob::new(
                Arc::clone(&self.auth_state),
                self.password.clone(),
                self.platform.clone(),
                self.email.clone(),
            );

            app.submit_job(job).unwrap();
        }

        if let Some(e) = self.error.as_ref() {
            ui.add_space(10.0);
            ui.label(e);
        }
    }

    fn should_close(&self, _: &App) -> bool {
        self.auth_success
    }
}

struct AuthenticateJob {
    state: Arc<Mutex<AuthenticateState>>,
    password: String,
    platform: wfm_rs::Platform,
    email: String,
}

impl AuthenticateJob {
    fn new(
        state: Arc<Mutex<AuthenticateState>>,
        password: String,
        platform: wfm_rs::Platform,
        email: String,
    ) -> Self {
        Self {
            state,
            password,
            platform,
            email,
        }
    }
}

impl Job for AuthenticateJob {
    fn run(&mut self, rt: &Runtime, _tx: &Sender<AppEvent>) -> anyhow::Result<()> {
        // TODO: use storage
        *self.state.lock() = AuthenticateState::Pending;

        let result = rt.block_on(User::login(
            &self.email,
            &self.password,
            &self.platform,
            "en",
        ));
        *self.state.lock() = AuthenticateState::Done(result);

        Ok(())
    }
}
