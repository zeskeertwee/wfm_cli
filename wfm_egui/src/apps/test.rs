use eframe::egui::{Context, Ui};

use crate::app::{App, AppWindow};

pub struct TestApp {}

impl AppWindow for TestApp {
    fn window_title(&self) -> String {
        "Test window".to_string()
    }

    fn update(&mut self, app: &App, _ctx: &Context, ui: &mut Ui) {
        ui.label("It's pretty empty here...");
    }
}
