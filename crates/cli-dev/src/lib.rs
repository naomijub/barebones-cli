pub mod error;
pub mod logging;
pub mod plugin;

pub mod prelude {
    pub use exitcode;

    pub use crate::{logging::LoggingConfig, plugin::*};
}
