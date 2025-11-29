use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error("failed to deserialize toml: {0}")]
    TomlDeserialization(#[from] toml::de::Error),
    #[error("failed to serialize toml: {0}")]
    TomlSerialization(#[from] toml::ser::Error),
    #[error("failed to create config: {0}")]
    Config(#[from] config::ConfigError),
    #[error("failed to acquire config lock: {0}")]
    LockPoison(String),
    #[error("config file watch failed")]
    ConfigWatcher,
}
