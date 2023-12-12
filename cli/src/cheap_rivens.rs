use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::time::Duration;
use colored::Colorize;
use leaky_bucket::RateLimiter;
use crate::util::data_path;
use serde::{Serialize, Deserialize};
use wfm_rs::response::{Auction, Auctions};
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

// TODO: optimize this to be smaller
#[derive(Serialize, Deserialize, Clone)]
struct RivenAuctionDataSet {
    data: Vec<Auction>,
    timestamp: u64,
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
        let data: Vec<RivenData> = serde_json::from_reader(&mut riven_data_file)?;
	riven_data = process_riven_data(&data)?;

	println!("[{}] Parsed from file", "INF".cyan());
        if unix_timestamp()? - riven_data.timestamp > WEEK_IN_SECONDS {
            println!("[{}] Refreshing riven data", "...".cyan());
            riven_data = download_riven_data(&mut riven_data_file).await?;
        }
    } else {
        let mut riven_data_file = File::create(&riven_data_file_path)?;
        riven_data = download_riven_data(&mut riven_data_file).await?;
    }

    println!("[{}] Riven data loaded!", "OK ".green());

    let mut data: RivenAuctionDataSet;

    let mut path = data_path()?;
    path.push(format!("{:?}_rivens.json", riven_type));
    if let Ok(mut file) = fs::OpenOptions::new().read(true).write(true).open(&path) { 
       	println!("[{}] Loading auction data from file", "INF".cyan());
	data = serde_json::from_reader(&file)?;

        if unix_timestamp()? - data.timestamp > 300 {
            println!("[{}] Not reloading auction data, less than 300s old!", "INF".cyan());
        } else {
            data = download_riven_auctions(&riven_data, &riven_type,&mut file).await?;
        }
    } else {
        data = download_riven_auctions(&riven_data, &riven_type, &mut File::create(&path)?).await?;
    }

    println!("\n[{}] Got {} auctions", "OK ".green(), data.data.len());

    println!("[{}] Analyzing auctions", "...".cyan());

    data.data.iter_mut().map(|v| if let Some(top) = v.top_bid { v.starting_price = top }).for_each(drop);

    data.data.sort_by(|a, b| match a.starting_price.cmp(&b.starting_price) {
        Ordering::Equal => if a.is_direct_sell && !b.is_direct_sell { Ordering::Less } else { Ordering::Equal },
        other => other,
    });

    let it: Vec<&Auction> = data.data.iter().filter(|v| v.owner.status == "ingame").collect();

    println!();
    for auction in it.iter().take(400).rev() {
        let status_text = match auction.is_direct_sell {
            true => "DIR".green(),
            false => "AUC".yellow(),
        };

        println!("[{}] {:<30} | {:>3}p | https://warframe.market/auction/{}", status_text, auction.item.name.cyan(), auction.starting_price, auction.id);
    }

    crate::util::press_enter_prompt();

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

async fn download_riven_auctions(riven_data: &ProcessedRivenDataSet, riven_type: &RivenType, file: &mut File) -> anyhow::Result<RivenAuctionDataSet> {
    let ratelimiter = RateLimiter::builder()
        .max(3)
        .initial(0)
        .refill(1)
        .interval(Duration::from_millis(400))
        .build();

    let rivens_of_type: Vec<String> = riven_data.data.iter().filter(|v| v.1 == riven_type).map(|v| v.0.to_owned()).collect();

    println!("{}", format!("\nNow the program will get riven auctions from warframe.market, this will take a while (est. {:.1}s)", rivens_of_type.len() as f64 / 3.0).cyan());

    let mut data = Vec::new();

    for (idx, weapon) in rivens_of_type.iter().enumerate() {
        ratelimiter.acquire_one().await;
        match wfm_rs::model::User::get_auctions_for_item(weapon.to_lowercase().as_str()).await {
            Ok(r) => {
                for i in r.auctions {
                    data.push(i);
                }

                println!("[{}] {:<30} ({:>3}/{:>3})", "OK ".green(), weapon.cyan(), idx + 1, rivens_of_type.len());
            },
            Err(e) => println!("[{}] Error getting {} riven auctions: {}", "ERR".red(), weapon.cyan(), e)
        }
    }

    let r = RivenAuctionDataSet {
        data,
        timestamp: unix_timestamp()?,
    };

    file.write_all(serde_json::to_string(&r)?.as_bytes())?;

    Ok(r)
}
