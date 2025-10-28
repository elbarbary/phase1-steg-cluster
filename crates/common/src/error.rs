use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Config error: {0}")]
    Config(String),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, Error>;
