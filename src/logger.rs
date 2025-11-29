use std::io::Write;

use cli_dev::logging::LoggingConfig;
use env_logger::Builder;
use log::LevelFilter;

use crate::APP_NAME;

pub fn initialize_logger(config: &LoggingConfig) {
    let mut builder = Builder::from_default_env();
    builder.format(|buf, record| {
        writeln!(
            buf,
            "[{}]: {} - {}",
            record.level(),
            record.target(),
            record.args()
        )
    });

    if config.verbose {
        builder.filter_level(LevelFilter::Trace);
    } else if config.quiet {
        builder.filter_level(LevelFilter::Error);
    } else {
        builder.filter_level(config.log_level);
    }
    builder.init();
}

pub fn log_error(message: impl Into<String>) {
    log::error!(target: APP_NAME, "{}", message.into());
}

pub fn log_warn(message: impl Into<String>) {
    log::warn!(target: APP_NAME, "{}", message.into());
}

pub fn log_info(message: impl Into<String>) {
    log::info!(target: APP_NAME, "{}", message.into());
}

pub fn log_debug(message: impl Into<String>) {
    log::debug!(target: APP_NAME, "{}", message.into());
}

pub fn log_trace(message: impl Into<String>) {
    log::trace!(target: APP_NAME, "{}", message.into());
}
