use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::RwLock;

use crate::APP_NAME;
use crate::config::data::MyConfig;
use config::{Config, File};

pub mod data;
pub mod watcher;

#[cfg(target_os = "windows")]
pub fn executable_location() -> PathBuf {
    dirs::home_dir()
        .expect("System should contain home dir")
        .join(APP_NAME)
        .join("barebones-cli.exe")
}

#[cfg(not(target_os = "windows"))]
pub fn executable_location() -> PathBuf {
    dirs::home_dir()
        .expect("System should contain home dir")
        .join(APP_NAME)
        .join("barebones-cli")
}

#[cfg(target_os = "windows")]
pub fn config_location() -> PathBuf {
    dirs::config_dir()
        .expect("System should contain config dir")
        .join(format!("{}.toml", APP_NAME))
}

#[cfg(not(target_os = "windows"))]
pub fn config_location() -> PathBuf {
    dirs::config_dir()
        .expect("System should contain config dir")
        .join(format!("{}.toml", APP_NAME))
}

fn settings() -> &'static RwLock<Config> {
    static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let settings = load();

        RwLock::new(settings)
    })
}

pub fn refresh() {
    *settings().write().unwrap() = load();
}

fn load() -> Config {
    let config_path = config_location();
    if !std::fs::exists(&config_path).unwrap_or_default() {
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .append(false)
            .open(config_path.to_string_lossy().trim())
            .expect("must be able to create config file");
        let config = MyConfig::default();
        let conf = toml::to_string(&config).unwrap();
        file.write_all(conf.as_bytes()).unwrap();
    }
    Config::builder()
        .add_source(File::with_name(config_path.to_string_lossy().trim()).required(true))
        .build()
        .unwrap()
}

pub fn show() {
    println!(
        " * Settings :: \n\x1b[31m{:?}\x1b[0m",
        settings()
            .read()
            .unwrap()
            .clone()
            .try_deserialize::<MyConfig>()
            .unwrap()
    );
}
