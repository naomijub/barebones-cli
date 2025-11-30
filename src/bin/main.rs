use barebones::{
    commands::Cli,
    config::{get_settings as config_show, refresh as config_refresh},
    logger::{self, initialize_logger, log_error},
    plugin_loader::PluginManager,
    updater::Updater,
};
use clap::Parser;
use crossbeam_channel::select;
use human_panic::{Metadata, setup_panic};

fn main() -> anyhow::Result<()> {
    let command = Cli::parse();
    initialize_logger(&command.logging);

    setup_panic!(
        Metadata::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
            .authors("Barebones CLI Corp <support@barebones-cli.corp>")
            .homepage("support.barebones-cli.corp")
            .support("- Open a support request by email to support@barebones-cli.corp")
    );
    let updater = Updater::new(
        "naomijub",      // GitHub username or org
        "barebones-cli", // Repository name
        "barebones-cli", // CLI name
    );

    if let Err(err) = updater.check_and_update(true) {
        log_error(err.to_string());
        return Err(anyhow::anyhow!("failed to auto-update"));
    }

    let mut manager = PluginManager::new();

    let crtlc_rx = barebones::signaling::ctrl_c::ctrlc_channel()?;
    let settings = config_show()?;

    if let Err(e) = manager.load_plugins_from_dir(&settings) {
        logger::log_error(format!("Error loading plugins: {}", e));
    }

    loop {
        select! {
            // For tasks that loop or long running taks
            recv(crtlc_rx) -> ctrl => {
                ctrl?.should_exit()
            },
            default => {
                config_refresh()?;
                match command.command {
                    barebones::commands::Commands::Greeter(ref trailing_args) => {
                        settings.contains_plugins(&"greeter".to_string());
                        manager.execute_plugin("greeter", trailing_args.args.clone())?;
                    },
                    barebones::commands::Commands::List => manager.list_plugins(),
                    barebones::commands::Commands::Show(ref name) => {
                        manager.show_help(&name.plugin)?;

                    },
                    barebones::commands::Commands::Version(ref version) => {
                        if version.version {
                            println!("version {}", self_update::cargo_crate_version!());
                        }
                    }
                }
            }
        }
    }
}
