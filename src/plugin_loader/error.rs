#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to load plugin: {0}")]
    Load(String),
    #[error("plugin not found: {0}")]
    NotFound(String),
    #[error("missing config dir")]
    MissingConfigDir,
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("execution error")]
    Execution,
}
