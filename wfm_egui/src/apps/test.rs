use eframe::egui::{CtxRef, Ui};

use crate::app::{App, AppWindow};

pub struct TestApp {}

impl AppWindow for TestApp {
    fn window_title(&self) -> String {
        "Test window".to_string()
    }

    fn update(&mut self, app: &App, _ctx: &CtxRef, ui: &mut Ui) {
        ui.image(app.placeholder_image(), (256.0, 256.0));
    }
}
