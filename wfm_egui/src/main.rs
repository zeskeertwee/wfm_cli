#![feature(type_name_of_val)]

use eframe;
use eframe::egui::vec2;

mod app;
mod apps;
mod background_jobs;
mod texture;
mod util;
mod worker;

fn main() {
    pretty_env_logger::init();
    let mut native_options = eframe::NativeOptions::default();
    native_options.initial_window_size = Some(vec2(800.0, 600.0));
    eframe::run_native(Box::new(app::App::default()), native_options);
}
