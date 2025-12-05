use std::process;

use crossbeam_channel::{Receiver, unbounded};
use tracing::{error, info};

use crate::APP_NAME;

pub struct CrtlCShouldExit(bool);

impl CrtlCShouldExit {
    pub fn should_exit(&self) {
        if self.0 {
            info!(target: APP_NAME, "crtl+c Signal received");
            process::exit(exitcode::UNAVAILABLE);
        }
    }
}

pub fn ctrlc_channel() -> Result<Receiver<CrtlCShouldExit>, ctrlc::Error> {
    let (crtlc_tx, crtlc_rx) = unbounded();
    ctrlc::set_handler(move || {
        if let Err(_err) = crtlc_tx.send(CrtlCShouldExit(true)) {
            error!(target: APP_NAME, "Could not send ctrl+c signal on channel.");
        }
    })?;

    Ok(crtlc_rx)
}
