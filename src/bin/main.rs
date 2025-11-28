use crossbeam_channel::select;

use human_panic::{Metadata, setup_panic};

use barebones::config::{config_location, show as config_show, watcher::watch as config_watch};

fn main() -> anyhow::Result<()> {
    setup_panic!(
        Metadata::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
            .authors("Barebones CLI Corp <support@barbones-cli.corp>")
            .homepage("support.barebones-cli.corp")
            .support("- Open a support request by email to support@barbones-cli.corp")
    );

    let crtlc_rx = barebones::signaling::ctrl_c::ctrlc_channel()?;
    config_show();
    let confic_rx = config_watch();
    loop {
        select! {
            recv(crtlc_rx) -> ctrl => {
                ctrl?.should_exit()
            },
            recv(confic_rx) -> config => {
                if let Ok(Ok(notify::Event {
                    kind: notify::event::EventKind::Modify(_),
                    ..
                })) = config {
                    println!(" * {} written; refreshing configuration ...", config_location().to_string_lossy());
                    barebones::config::refresh();
                    barebones::config::show();
                }
            },
            default => {
                eprintln!("waiting something");
            }
        }
    }

    Ok(())
}
