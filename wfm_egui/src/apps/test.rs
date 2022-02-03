use crate::app::{App, AppWindow};
use eframe::egui::{CtxRef, Ui};

pub struct TestApp {}

impl AppWindow for TestApp {
    fn window_title(&self) -> &str {
        "Test window"
    }

    fn update(&mut self, app: &App, ctx: &CtxRef, ui: &mut Ui) {
        ui.image(app.placeholder_image(), (256.0, 256.0));
    }

    fn should_close(&self, _: &App) -> bool {
        false
    }
}
