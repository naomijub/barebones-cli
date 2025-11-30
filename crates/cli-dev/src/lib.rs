pub mod error;
pub mod logging;
pub mod plugin;

pub mod prelude {
    pub use exitcode;

    pub use crate::{logging::LoggingConfig, plugin::*};
}

#[deprecated(note = "Use cli_dev::log_debug!() instead of log::debug!() directly")]
pub use log::debug;
#[deprecated(note = "Use cli_dev::log_error!() instead of log::error!() directly")]
pub use log::error;
#[deprecated(note = "Use cli_dev::log_info!() instead of log::info!() directly")]
pub use log::info;
#[deprecated(note = "Use cli_dev::log_trace!() instead of log::trace!() directly")]
pub use log::trace;
#[deprecated(note = "Use cli_dev::log_warn!() instead of log::warn!() directly")]
pub use log::warn;
