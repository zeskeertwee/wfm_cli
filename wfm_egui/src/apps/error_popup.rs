use eframe::egui::{Context, Rgba, RichText, Ui};
use crate::app::{App, AppWindow};

pub struct ErrorPopup {
    title: String,
    message: String,
    ok: bool,
}

impl ErrorPopup {
    pub fn new(title: &str, message: &str) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            ok: false,
        }
    }
}

impl AppWindow for ErrorPopup {
    fn window_title(&self) -> String {
        self.title.clone()
    }

    fn show_close_button(&self) -> bool {
        true
    }

    fn update(&mut self, _app: &App, _ctx: &Context, ui: &mut Ui) {
        ui.add_space(25.0);
        ui.label(RichText::new("!  Error:").color(Rgba::from_rgb(229.0 / 255.0, 39.0 / 255.0, 21.0 / 255.0)).monospace());
        ui.horizontal(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(&self.message).monospace());
            });
        });
        ui.add_space(25.0);
        if ui.button("Ok").clicked() {
              self.ok = true;
        }
    }

    fn should_close(&self, app: &App) -> bool {
        self.ok
    }
}