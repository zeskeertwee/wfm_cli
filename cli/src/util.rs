use std::path::PathBuf;
use std::time::SystemTime;
use anyhow::Result;
use home;
use crate::{
    DATA_SCREENSHOT_DIR,
    DATA_PATH_SUFFIX,
    DATA_CONFIG_FILE,
};

pub fn data_path() -> Result<PathBuf> {
    let mut home_dir = match home::home_dir() {
        Some(x) => x,
        None => anyhow::bail!("Failed to find home directory!"),
    };

    home_dir.push(DATA_PATH_SUFFIX);

    Ok(home_dir)
}

pub fn screenshot_path() -> Result<PathBuf> {
    let mut data_path = data_path()?;
    data_path.push(DATA_SCREENSHOT_DIR);

    Ok(data_path)
}

pub fn config_path() -> Result<PathBuf> {
    let mut data_path = data_path()?;
    data_path.push(DATA_CONFIG_FILE);

    Ok(data_path)
}

pub fn unix_timestamp() -> Result<u64> {
    Ok(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs())
}

// https://stackoverflow.com/questions/34837011/how-to-clear-the-terminal-screen-in-rust-after-a-new-line-is-printed
pub fn clear_terminal() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}