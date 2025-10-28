use thiserror::Error;

#[derive(Debug, Error)]
pub enum StegoError {
    #[error("Capacity exceeded: need {needed} bytes, available {available} bytes")]
    CapacityExceeded { needed: u64, available: u64 },

    #[error("Invalid magic number: expected 0x53544547, got {0:#x}")]
    InvalidMagic(u32),

    #[error("CRC mismatch: expected {expected:#x}, got {actual:#x}")]
    CrcMismatch { expected: u32, actual: u32 },

    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Compression error: {0}")]
    Compression(std::io::Error),

    #[error("Invalid cover image: {0}")]
    InvalidCover(String),

    #[error("Extraction failed: {0}")]
    ExtractionFailed(String),
}

pub type Result<T> = std::result::Result<T, StegoError>;
