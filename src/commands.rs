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
    Greeter(Args),

    /// List Plugins
    List,

    Show(Helper),

    /// Version
    Version(Version),
}

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}

#[derive(Debug, Parser)]
pub struct Helper {
    #[arg(value_name = "PLUGIN_NAME")]
    pub plugin: String,
}

#[derive(Debug, Parser)]
pub struct Version {
    #[arg(short, long)]
    pub version: bool,
}
