#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to configure Log Level Filter: `{0}`")]
    LogLevelRead(String),
}
