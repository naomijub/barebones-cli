use std::str::FromStr;

use abi_stable::std_types::RString;
use clap::Parser;

use log::LevelFilter;

use crate::{error::Error, plugin::PluginModRef};

#[derive(Parser, Debug)]
pub struct LoggingConfig {
    /// Hides test progression. Defaults to 'false'
    #[arg(long, short, value_name = "QUIET", group = "verbosity", global = true)]
    pub quiet: bool,

    /// Enable verbose logging. Defaults to 'false'.
    ///
    /// NOTE: Overrides log-level
    #[arg(
        long,
        short,
        value_name = "VERBOSE",
        group = "verbosity",
        global = true
    )]
    pub verbose: bool,

    /// Sets the log-level of the CLI. Defaults to 'Warn'.
    #[arg(long, short, value_name = "LEVEL", value_parser = log_in_range, default_value_t = LevelFilter::Warn, global = true)]
    pub log_level: LevelFilter,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            quiet: false,
            verbose: false,
            log_level: LevelFilter::Warn,
        }
    }
}

fn log_in_range(s: &str) -> Result<LevelFilter, Error> {
    LevelFilter::from_str(s).map_err(|_| Error::LogLevelRead(s.to_uppercase()))
}

pub fn log_error(plugin: PluginModRef, message: impl Into<String>) {
    let binding = (plugin.get_info())
        .map(|f| f().name)
        .unwrap_or_else(|| RString::from_str("UNKNOWN").unwrap());
    let name = binding.as_str();
    log::error!(target: name, "{}", message.into());
}

pub fn log_warn(plugin: PluginModRef, message: impl Into<String>) {
    let binding = (plugin.get_info())
        .map(|f| f().name)
        .unwrap_or_else(|| RString::from_str("UNKNOWN").unwrap());
    let name = binding.as_str();
    log::warn!(target: name, "{}", message.into());
}

pub fn log_info(plugin: PluginModRef, message: impl Into<String>) {
    let binding = (plugin.get_info())
        .map(|f| f().name)
        .unwrap_or_else(|| RString::from_str("UNKNOWN").unwrap());
    let name = binding.as_str();
    log::info!(target: name, "{}", message.into());
}

pub fn log_debug(plugin: PluginModRef, message: impl Into<String>) {
    let binding = (plugin.get_info())
        .map(|f| f().name)
        .unwrap_or_else(|| RString::from_str("UNKNOWN").unwrap());
    let name = binding.as_str();
    log::debug!(target: name, "{}", message.into());
}

pub fn log_trace(plugin: PluginModRef, message: impl Into<String>) {
    let binding = (plugin.get_info())
        .map(|f| f().name)
        .unwrap_or_else(|| RString::from_str("UNKNOWN").unwrap());
    let name = binding.as_str();
    log::trace!(target: name, "{}", message.into());
}
