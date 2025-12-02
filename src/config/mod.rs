use std::{
    io::Write,
    path::PathBuf,
    sync::{OnceLock, RwLock},
    time::SystemTime,
};

use config::{Config, File};

use crate::{
    APP_NAME,
    config::{data::MyConfig, error::Error},
    logger::log_debug,
};

pub mod data;
pub mod error;

static LAST_MODIFIED: OnceLock<RwLock<SystemTime>> = OnceLock::new();

#[cfg(target_os = "windows")]
pub fn executable_location() -> PathBuf {
    dirs::home_dir()
        .expect("System should contain home dir")
        .join(format!(".{APP_NAME}"))
        .join("barebones-cli.exe")
}

#[cfg(not(target_os = "windows"))]
pub fn executable_location() -> PathBuf {
    dirs::home_dir()
        .expect("System should contain home dir")
        .join(format!(".{APP_NAME}"))
        .join("barebones-cli")
}

pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("System should contain home dir")
        .join(format!(".{APP_NAME}"))
}

pub fn config_location() -> PathBuf {
    dirs::home_dir()
        .expect("System should contain home dir")
        .join(format!(".{APP_NAME}"))
        .join("config.toml")
}

fn settings() -> &'static RwLock<Config> {
    static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let settings = load().unwrap();

        RwLock::new(settings)
    })
}

pub fn refresh() -> Result<(), Error> {
    let config_path = config_location();
    if let (Ok(time), old) = (
        std::fs::metadata(config_path)?.modified(),
        LAST_MODIFIED.get_or_init(|| RwLock::new(SystemTime::now())),
    ) {
        let mut should_update = false;
        if let Ok(old) = old.read()
            && *old != time
        {
            should_update = true;
        }
        if should_update && let Ok(mut old) = old.write() {
            *old = time;
            let new_config = load()?;
            log_debug(format!(
                "Updated Config: {}",
                toml::to_string_pretty(&new_config.clone().try_deserialize::<MyConfig>()?)?
            ));
            *settings()
                .write()
                .map_err(|err| Error::LockPoison(err.to_string()))? = new_config;
        }
    }

    Ok(())
}

fn load() -> Result<Config, Error> {
    let config_path = config_location();
    create_config(&config_path)?;

    Ok(Config::builder()
        .add_source(File::with_name(config_path.to_string_lossy().trim()).required(true))
        .build()?)
}

fn create_config(config_path: &PathBuf) -> Result<(), Error> {
    if !std::fs::exists(config_path).unwrap_or_default() {
        std::fs::create_dir_all(config_dir())?;
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .append(false)
            .open(config_path.to_string_lossy().trim())?;
        let config = MyConfig::default();
        let conf = toml::to_string(&config)?;
        file.write_all(conf.as_bytes())?;
        LAST_MODIFIED.get_or_init(|| RwLock::new(SystemTime::now()));
    }

    Ok(())
}

pub fn get_settings() -> Result<MyConfig, Error> {
    let config = settings()
        .read()
        .map_err(|err| Error::LockPoison(err.to_string()))?
        .clone()
        .try_deserialize::<MyConfig>()?;
    Ok(config)
}
