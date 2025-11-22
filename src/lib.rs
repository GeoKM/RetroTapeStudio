pub mod backup;
pub mod core;
pub mod gui;
pub mod log;
pub mod summary;
pub mod tap;
pub mod utils;

#[derive(Debug, thiserror::Error)]
pub enum TapeError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
}

pub type TapeResult<T> = Result<T, TapeError>;
