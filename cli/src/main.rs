use std::{time::Duration, thread, fs};
use ocr::OCREngine;
use tokio;
use device_query::{DeviceQuery, DeviceState, Keycode};
use screenshot_rs;
use util::{clear_terminal, screenshot_path, unix_timestamp};
use wfm_rs::response::ShortItem;
use wfm_rs::User;
use anyhow::Result;
use colored::*;

mod config;
mod ocr;
mod util;

const DATA_PATH_SUFFIX: &str = ".wfm_cli/";
const DATA_SCREENSHOT_DIR: &str = "screenshots/";
const DATA_CONFIG_FILE: &str = "config.wfm.json";
const ITEMS_CACHE_EXPIRY_S: u64 = 24 * 60 * 60;
const RESULT_COLORS: [Color; 4] = [
    Color::TrueColor { r: 0,   g: 255, b: 8 },
    Color::TrueColor { r: 255, g: 174, b: 9 },
    Color::TrueColor { r: 255, g: 99,  b: 9},
    Color::TrueColor { r: 255, g: 12,  b: 9}
];

// TODO:
// - silence tesseract dpi complaining
// - release wfm_rs
// - release cli

#[cfg(target_os = "windows")]
std::compile_error!("Windows is not supported!");

#[tokio::main]
async fn main() {
    let config = config::run().await.unwrap();
    let user = config.user();
    let device = DeviceState::new();
    let engine = OCREngine::new(config.items);
    println!("You may now press '~' whenever you get to the relic reward screen");

    loop {
        let keys: Vec<Keycode> = device.get_keys();
        if keys.contains(&Keycode::Grave) {
            println!("Scanning...");
            let mut screenshot_path = screenshot_path().unwrap();
            screenshot_path.push(format!("{}.png", unix_timestamp().unwrap()));
            let screenshot_path_str = screenshot_path.to_string_lossy().to_string();
            screenshot_rs::screenshot_window(screenshot_path_str.clone());
            let items = engine.ocr(&screenshot_path_str).unwrap();
            fs::remove_file(screenshot_path).unwrap();
            
            let mut all_item_stats = Vec::new();
            
            for item in items {
                all_item_stats.push(get_item_info(&item, &user).await.unwrap());
            }
            
            all_item_stats.sort_by(|a, b| a.avg_price.partial_cmp(&b.avg_price).unwrap());
            let all_item_stats: Vec<&ItemStats> = all_item_stats.iter().rev().collect();

            clear_terminal();
            
            for (idx, item) in all_item_stats.iter().enumerate() {
                let msg = format!("{} | {:.1} platinum average | {:.0} sold in the last 48 hours", item.item.item_name, item.avg_price, item.volume);
                println!("{}", msg.color(RESULT_COLORS[idx]));
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    
}

#[derive(Clone)]
struct ItemStats {
    volume: f32,
    avg_price: f32,
    item: ShortItem,
}

async fn get_item_info(item: &ShortItem, user: &User) -> Result<ItemStats> {
    let statistics = user.get_item_market_statistics(item).await?;

    let last_stats = &statistics.statistics_closed._48_hours;
    let avg_price: f32 = last_stats.iter().map(|x| x.avg_price).sum::<f32>() / last_stats.len() as f32;
    let volume: f32 = last_stats.iter().map(|x| x.volume).sum();

    Ok(ItemStats {
        volume,
        avg_price,
        item: item.clone(),
    })
}