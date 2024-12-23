use document::{BuiltinFont, FontId};
use thiserror::Error;

pub mod document;
pub mod graphics;
pub mod page;
pub mod time_impl;
pub mod units;
pub(crate) mod utils;
#[derive(Debug, Error)]
pub enum TuxPdfError {
    #[error("Font not registered: {0:?}")]
    FontNotRegistered(FontId),
    #[error("Builtin font not registered: {0:?}")]
    BuiltinFontNotRegistered(BuiltinFont),
    #[error("No pages created")]
    NoPagesCreated,
    #[error("Invalid object id: {0}")]
    InvalidObjectId(String),
    #[error("ObjectID already exists: {0}")]
    ObjectCollectionError(String),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    FontParseError(#[from] ttf_parser::FaceParsingError),
    #[error(transparent)]
    TableError(#[from] graphics::table::TableError),
}

pub type TuxPdfResult<T> = Result<T, TuxPdfError>;

#[cfg(test)]
pub(crate) mod tests {
    use std::path::PathBuf;
    /// Returns the path to the fonts directory in the tests directory
    pub fn test_fonts_directory() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fonts")
    }
    pub fn does_end_with_ttf(path: &std::path::Path) -> bool {
        path.extension().map_or(false, |ext| ext == "ttf")
    }
}
