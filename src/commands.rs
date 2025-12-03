use clap::{Parser, Subcommand};
use cli_dev::prelude::LoggingConfig;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[clap(flatten)]
    pub logging: LoggingConfig,

    #[clap(flatten)]
    pub accepter: GithubAccepter,

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

    /// Manually update CLI
    Update,
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
pub struct Version {}

#[derive(Parser, Debug)]
pub struct GithubAccepter {
    /// Should auto accept download request
    #[arg(long, default_value_t = false)]
    pub accept: bool,
}
