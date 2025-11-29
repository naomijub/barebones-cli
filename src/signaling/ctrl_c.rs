use std::process;

use crossbeam_channel::{Receiver, unbounded};

use crate::logger::{log_error, log_info};

pub struct CrtlCShouldExit(bool);

impl CrtlCShouldExit {
    pub fn should_exit(&self) {
        if self.0 {
            log_info("crtl+c Signal received");
            process::exit(exitcode::UNAVAILABLE);
        }
    }
}

pub fn ctrlc_channel() -> Result<Receiver<CrtlCShouldExit>, ctrlc::Error> {
    let (crtlc_tx, crtlc_rx) = unbounded();
    ctrlc::set_handler(move || {
        if let Err(_err) = crtlc_tx.send(CrtlCShouldExit(true)) {
            log_error("Could not send ctrl+c signal on channel.");
        }
    })?;

    Ok(crtlc_rx)
}
