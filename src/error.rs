pub type Result<T> = std::result::Result<T, Error>;

#[derive(PartialEq, Eq, Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to open  file: {0}")]
    InalivdFile(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}
