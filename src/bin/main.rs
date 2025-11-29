use clap::Parser;
use crossbeam_channel::select;

use human_panic::{Metadata, setup_panic};

use barebones::{
    commands::Cli,
    config::{get_settings as config_show, refresh as config_refresh},
    logger::{self, initialize_logger},
    plugin_loader::PluginManager,
};

fn main() -> anyhow::Result<()> {
    let command = Cli::parse();
    initialize_logger(&command.logging);

    setup_panic!(
        Metadata::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
            .authors("Barebones CLI Corp <support@barebones-cli.corp>")
            .homepage("support.barebones-cli.corp")
            .support("- Open a support request by email to support@barebones-cli.corp")
    );
    let mut manager = PluginManager::new();

    let crtlc_rx = barebones::signaling::ctrl_c::ctrlc_channel()?;
    let settings = config_show()?;

    if let Err(e) = manager.load_plugins_from_dir(&settings) {
        logger::log_error(format!("Error loading plugins: {}", e));
    }

    loop {
        select! {
            recv(crtlc_rx) -> ctrl => {
                ctrl?.should_exit()
            },
            default => {
                match command.command {
                    barebones::commands::Commands::Greeter => {
                        manager.execute_plugin("greeter", vec!["Julia".to_string()])?;
                    },
                    barebones::commands::Commands::List => manager.list_plugins(),
                    barebones::commands::Commands::Wait => {
                        config_refresh()?
                    },
                }
            }
        }
    }
}
