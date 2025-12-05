use std::str::FromStr;

use clap::Parser;
use tracing::level_filters::LevelFilter;
pub use tracing::{
    debug, debug_span, error, error_span, event, info, info_span, span, trace, trace_span, warn,
    warn_span,
};

use crate::error::Error;

#[derive(Parser, Debug)]
pub struct LoggingConfig {
    /// Prints only output and stderr. Defaults to 'false'
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

    /// Sets the log-level of the CLI. Defaults to 'INFO'.
    #[arg(long, short, value_name = "LEVEL", value_parser = log_in_range, default_value_t = LevelFilter::INFO, global = true)]
    pub log_level: LevelFilter,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            quiet: false,
            verbose: false,
            log_level: LevelFilter::INFO,
        }
    }
}

fn log_in_range(s: &str) -> Result<LevelFilter, Error> {
    LevelFilter::from_str(s).map_err(|_| Error::LogLevelRead(s.to_uppercase()))
}
