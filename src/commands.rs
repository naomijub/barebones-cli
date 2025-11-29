use clap::{Parser, Subcommand};
use cli_dev::prelude::LoggingConfig;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[clap(flatten)]
    pub logging: LoggingConfig,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Greeter Plugin
    Greeter,

    /// List Plugins
    List,

    Wait,
}
