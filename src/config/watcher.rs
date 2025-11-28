use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

use cli_dev::adapters::crossbeam::to_crossbeam_receiver;
use crossbeam_channel::Receiver;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::config::config_location;

pub fn watch() -> Receiver<Result<notify::Event, notify::Error>> {
    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher: RecommendedWatcher = Watcher::new(
        tx,
        notify::Config::default().with_poll_interval(Duration::from_secs(2)),
    )
    .unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher
        .watch(Path::new(&config_location()), RecursiveMode::NonRecursive)
        .unwrap();

    to_crossbeam_receiver(rx)
}
