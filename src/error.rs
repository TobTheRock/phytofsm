pub type Result<T> = std::result::Result<T, Error>;

#[derive(PartialEq, Eq, Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to open file: {0}")]
    InvalidFile(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Parsed FSM representation is invalid: {0}")]
    InvalidFsm(String),
}
