use document::{FontRef, ResourceNotRegistered};
use thiserror::Error;

pub mod document;
pub mod graphics;
pub mod page;
pub mod time_impl;
pub mod units;
pub(crate) mod utils;
#[derive(Debug, Error)]
pub enum TuxPdfError {
    #[error(transparent)]
    ResourceNotRegistered(#[from] ResourceNotRegistered),
    #[error("Invalid Reference. Expected reference to {0}")]
    InvalidReference(&'static str),
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
    #[error(transparent)]
    ImageCrateError(#[from] image::ImageError),
    #[error("Unsupported image color type: {0:?}")]
    UnsupportedImageColorType(image::ColorType),
}
impl From<FontRef> for TuxPdfError {
    fn from(font_ref: FontRef) -> Self {
        ResourceNotRegistered::from(font_ref).into()
    }
}
pub type TuxPdfResult<T> = Result<T, TuxPdfError>;

#[cfg(test)]
pub(crate) mod tests {
    include!("../tests/test_utils_external.rs");
}
