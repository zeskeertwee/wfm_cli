use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::time::Instant;

use eframe::egui::{CtxRef, Ui};
use eguikit::spinner::Style;
use eguikit::Spinner;

use crate::app::{App, AppWindow};

pub struct SpinnerPopup {
    pending_storage_key: String,
    finished_show_for_s: f64,
    text: String,
    finished_text: String,
    title: String,
    show_finished_text: AtomicBool,
}

impl AppWindow for SpinnerPopup {
    fn window_title(&self) -> &str {
        &self.title
    }

    fn show_close_button(&self) -> bool {
        false
    }

    fn update(&mut self, _app: &App, _ctx: &CtxRef, ui: &mut Ui) {
        ui.add_space(50.0);
        ui.vertical_centered(|ui| {
            ui.label(match self.show_finished_text.load(Ordering::Relaxed) {
                true => &self.finished_text,
                false => &self.text,
            });

            if !self.show_finished_text.load(Ordering::Relaxed) {
                ui.add(Spinner::default().style(Style::Dots));
            }
        });
        ui.add_space(50.0);
    }

    fn should_close(&self, app: &App) -> bool {
        match app.get_from_storage::<Instant, _, _>(
            &self.get_finished_show_until_instant_key(),
            |instant| {
                match instant {
                    // true if the spinner should still be visible
                    Some(instant) => {
                        Some(instant.elapsed() < Duration::from_secs_f64(self.finished_show_for_s))
                    }
                    None => None,
                }
            },
        ) {
            Some(b) => return !b,
            None => (),
        }

        if !app.present_in_storage(&self.pending_storage_key) {
            // finished, close
            app.remove_from_storage(&self.pending_storage_key);
            if self.finished_show_for_s <= 0.0 {
                log::trace!("Not configured to show finished popup after finishing");
                return true;
            }

            log::trace!(
                "Configured to show finished popup after finishing for {}s",
                self.finished_show_for_s
            );
            app.insert_into_storage(&self.get_finished_show_until_instant_key(), Instant::now());
            self.show_finished_text.store(true, Ordering::Relaxed);
            return false;
        }
        false
    }
}

impl SpinnerPopup {
    pub fn new(
        app: &App,
        title: &str,
        text: &str,
        finished_text: Option<&str>,
        storage_id: &str,
        show_for_s_after_finished: f64,
    ) {
        let spinner = SpinnerPopup {
            pending_storage_key: storage_id.to_string(),
            text: text.to_string(),
            title: title.to_string(),
            finished_show_for_s: show_for_s_after_finished,
            finished_text: finished_text
                .unwrap_or(&format!("{} finished", text))
                .to_string(),
            show_finished_text: AtomicBool::new(false),
        };

        app.queue_window_spawn(spinner);
    }

    fn get_finished_show_until_instant_key(&self) -> String {
        format!("{}-show-until", self.pending_storage_key)
    }
}
