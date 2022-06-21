use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::time::Duration;
use colored::Colorize;
use leaky_bucket::RateLimiter;
use crate::util::data_path;
use serde::{Serialize, Deserialize};
use crate::unix_timestamp;

const WEEK_IN_SECONDS: u64 = 60 * 60 * 24 * 7;

// if this seems sketchy, i don't blame you.
// you may be wondering how i found it, and i found it on the warframe wiki:
// https://warframe.fandom.com/wiki/Riven_Mods#Trade_Data
const RIVEN_DATA_URL: &str = "http://n9e5v4d8.ssl.hwcdn.net/repos/weeklyRivensPC.json";

// this scans the market for cheap rivens of a specific type (rifle/melee/shotgun etc.)
// so you can transmute them into a veiled one and (hopefully) make a profit

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ProcessedRivenDataSet {
    data: HashMap<String, RivenType>,
    timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct RivenData {
    #[serde(rename(deserialize = "itemType"))]
    riven_type: RivenType,
    #[serde(rename(deserialize = "compatibility"))]
    weapon: Option<String>,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
enum RivenType {
    #[serde(alias = "Melee Riven Mod")]
    Melee,
    #[serde(alias = "Pistol Riven Mod")]
    Pistol,
    #[serde(alias = "Rifle Riven Mod")]
    Rifle,
    #[serde(alias = "Kitgun Riven Mod")]
    Kitgun,
    #[serde(alias = "Zaw Riven Mod")]
    Zaw,
    #[serde(alias = "Shotgun Riven Mod")]
    Shotgun,
    #[serde(alias = "Archgun Riven Mod")]
    Archgun,
}

impl RivenType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "melee" => Some(RivenType::Melee),
            "pistol" => Some(RivenType::Pistol),
            "rifle" => Some(RivenType::Rifle),
            "kitgun" => Some(RivenType::Kitgun),
            "zaw" => Some(RivenType::Zaw),
            "shotgun" => Some(RivenType::Shotgun),
            "archgun" => Some(RivenType::Archgun),
            _ => None,
        }
    }
}

pub async fn run(type_str: &str) -> anyhow::Result<()> {
    let riven_type = match RivenType::from_str(type_str) {
        Some(t) => t,
        None => {
            println!("[{}] {} is not a valid riven type", "ERR".red(), type_str);
            return Ok(());
        }
    };

    let mut riven_data_file_path = data_path()?;
    riven_data_file_path.push("riven_data.json");

    let mut riven_data: ProcessedRivenDataSet;

    if let Ok(mut riven_data_file) = fs::OpenOptions::new().read(true).write(true).open(&riven_data_file_path) {
        riven_data = serde_json::from_reader(&mut riven_data_file)?;

        if unix_timestamp()? - riven_data.timestamp > WEEK_IN_SECONDS {
            println!("[{}] Refreshing riven data", "...".cyan());
            riven_data = download_riven_data(&mut riven_data_file).await?;
        }
    } else {
        let mut riven_data_file = File::create(&riven_data_file_path)?;
        riven_data = download_riven_data(&mut riven_data_file).await?;
    }

    println!("[{}] Riven data loaded!", "OK ".green());

    let mut ratelimiter = RateLimiter::builder()
        .max(3)
        .initial(0)
        .refill(3)
        .interval(Duration::from_secs(1))
        .build();

    let rivens_of_type = riven_data.data.iter().filter(|v| v.1 == &riven_type).count();

    println!("{}", format!("\nNow the program will get riven auctions from warframe.market, this will take a while (est. {:.1}s)", rivens_of_type as f64 / 3.0).cyan());
    //wfm_rs::model::User::get_auctions_for_item();

    Ok(())
}

async fn download_riven_data(file: &mut File) -> anyhow::Result<ProcessedRivenDataSet> {
    println!("[{}] Downloading riven data", "...".cyan());
    let r: String = reqwest::get(RIVEN_DATA_URL).await?.text().await?;
    let data: Vec<RivenData> = serde_json::from_str(&r)?;
    let processed = process_riven_data(&data)?;

    file.write_all(serde_json::to_string(&processed)?.as_bytes())?;

    Ok(processed)
}

fn process_riven_data(data: &Vec<RivenData>) -> anyhow::Result<ProcessedRivenDataSet> {
    let mut riven_data: HashMap<String, RivenType> = HashMap::new();

    for riven in data {
        if let Some(weapon) = &riven.weapon {
            if riven_data.get(weapon).is_none() {
                riven_data.insert(weapon.clone(), riven.riven_type);
            }
        }
    }

    Ok(ProcessedRivenDataSet {
        data: riven_data,
        timestamp: unix_timestamp()?,
    })
}