use document::{BuiltinFont, FontId, FontRef};
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
impl From<FontRef> for TuxPdfError {
    fn from(font_ref: FontRef) -> Self {
        match font_ref {
            FontRef::External(font_id) => TuxPdfError::FontNotRegistered(font_id),
            FontRef::Builtin(builtin_font) => TuxPdfError::BuiltinFontNotRegistered(builtin_font),
        }
    }
}
pub type TuxPdfResult<T> = Result<T, TuxPdfError>;

#[cfg(test)]
pub(crate) mod tests {
    include!("../tests/test_utils_external.rs");
}
