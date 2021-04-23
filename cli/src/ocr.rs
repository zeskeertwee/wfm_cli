use std::{fs, thread};
use std::sync::{Arc, RwLock};
use crossbeam_channel::{Sender, Receiver, unbounded};
use tesseract;
use image::{DynamicImage, GenericImage, GenericImageView, Pixel};
use anyhow::Result;
use home;
use wfm_rs::response::ShortItem;
use levenshtein::levenshtein;
use crate::{
    DATA_PATH_SUFFIX,
    DATA_SCREENSHOT_DIR,
    util::unix_timestamp,
};

const IMG_MAX_WHITE_DEV: f32 = 45.0;
const ITEM_CROP_SIZE: [u32; 2] = [250, 50];
const ITEM_CROP_COORDS: [[u32; 2]; 4] = [
    [470, 410],
    [720, 410],
    [960, 410],
    [1200, 410]
];

pub struct OCREngine {
    tx: [Sender<DynamicImage>; 4],
    rx: Receiver<ShortItem>,
}

impl OCREngine {
    pub fn new(items: Vec<ShortItem>) -> OCREngine {
        let img_channels: [(Sender<DynamicImage>, Receiver<DynamicImage>); 4] = [
            unbounded(),
            unbounded(),
            unbounded(),
            unbounded(),
        ];

        let (ret_channel_tx, ret_channel_rx) = unbounded::<ShortItem>();
        let items = Arc::new(RwLock::new(items));

        for i in 0..4 {
            let thread_rx = img_channels[i].1.clone();
            let thread_tx = ret_channel_tx.clone();
            let thread_items = items.clone();
            let _ = thread::spawn(move || {
                let rx = thread_rx;
                let tx = thread_tx;
                let items = thread_items;
                let idx = i;
                let mut ts = tesseract::Tesseract::new_with_oem(None, Some("eng"), tesseract::OcrEngineMode::Default).unwrap();
                let mut data_path = home::home_dir().unwrap();
                data_path.push(DATA_PATH_SUFFIX);
                data_path.push(DATA_SCREENSHOT_DIR);
                
                loop {
                    let mut img = match rx.recv() {
                        Ok(x) => x,
                        Err(e) => {
                            eprintln!("Error in ocr worker: {}", e);
                            continue;
                        }
                    };

                    img = remove_not_white(&img, IMG_MAX_WHITE_DEV);
                    let mut img_path = data_path.clone();
                    img_path.push(format!("{}_{}.png", unix_timestamp().unwrap(), idx));
                    img.save(&img_path).unwrap();
                    let img_path_str = format!("{:?}", img_path).replace(r#"""#, "");
                    ts = ts.set_image(&img_path_str).unwrap().recognize().unwrap();
                    let raw_ocr = ts.get_text().unwrap();
                    fs::remove_file(img_path).unwrap();
                    let closest = find_closest_levenshtein_match(&items.read().unwrap(), &raw_ocr);
                    tx.send(closest).unwrap();
                }
            });
        }

        OCREngine {
            tx: [
                img_channels[0].0.clone(),
                img_channels[1].0.clone(),
                img_channels[2].0.clone(),
                img_channels[3].0.clone(),
            ],
            rx: ret_channel_rx,
        }
    }

    pub fn ocr(&self, path: &str) -> Result<Vec<ShortItem>> {
        let img = image::open(path)?;

        for i in 0..4 {
            let cropped = img.crop_imm(ITEM_CROP_COORDS[i][0], ITEM_CROP_COORDS[i][1], ITEM_CROP_SIZE[0], ITEM_CROP_SIZE[1]);
            self.tx[i].send(cropped)?;
        }

        let mut results = Vec::new();

        for _ in 0..4 {
            results.push(self.rx.recv()?);
        }

        Ok(results)
    }
}

// https://github.com/WFCD/WFinfo/blob/a7d4b8311564807cf384495441a18c56f63f7eb1/WFInfo/Data.cs#L830
fn find_closest_levenshtein_match(items: &Vec<ShortItem>, target: &str) -> ShortItem {
    let mut lowest_levenshtein = 9999;
    let mut lowest_item = None;
    
    for item in items {
        let diff = levenshtein(target, &item.item_name);
        if diff < lowest_levenshtein {
            lowest_levenshtein = diff;
            lowest_item = Some(item);
        }
    }

    lowest_item.unwrap().clone()
}

fn remove_not_white(img: &DynamicImage, max_dev: f32) -> DynamicImage {
    let mut result = img.clone();
    for pix in img.pixels() {
        let x = pix.0;
        let y = pix.1;
        let color = pix.2;

        if pixel_dev(color) > max_dev {
            result.put_pixel(x, y, Pixel::from_channels(0, 0, 0, 255));
        } else {
            result.put_pixel(x, y, Pixel::from_channels(255, 255, 255, 255));
        }
    }

    result
}

fn pixel_dev(pixel: image::Rgba<u8>) -> f32 {
    (255.0 - pixel[0] as f32) +
    (255.0 - pixel[1] as f32) +
    (255.0 - pixel[2] as f32)
}