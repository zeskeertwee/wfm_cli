use std::fs::File;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;

pub fn clone_option_with_inner_ref<T: Clone>(v: Option<&T>) -> Option<T> {
    v.map(|v| v.clone())
}

pub fn get_storage_path(file_name: &str) -> PathBuf {
    let mut dir = eframe::storage_dir("wim-warframeinventorymanager").unwrap();
    dir.push(file_name);
    dir
}

pub fn get_storage_file(file_name: &str) -> Result<File> {
    Ok(File::open(get_storage_path(file_name))?)
}

pub fn create_storage_file(file_name: &str) -> Result<File> {
    Ok(File::create(get_storage_path(file_name))?)
}

pub fn get_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}