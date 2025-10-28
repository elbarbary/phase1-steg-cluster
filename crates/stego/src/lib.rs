pub mod error;
pub mod lsb;
pub mod utils;

pub use error::{Result, StegoError};
pub use lsb::{embed, extract, CoverInfo};
pub use utils::{generate_cover_image, get_mime_type};
