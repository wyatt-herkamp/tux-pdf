use document::{FontRef, ResourceNotRegistered};
use graphics::layouts::LayoutError;
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
    #[error(transparent)]
    LayoutError(#[from] LayoutError),
}
impl From<FontRef> for TuxPdfError {
    fn from(font_ref: FontRef) -> Self {
        ResourceNotRegistered::from(font_ref).into()
    }
}
pub type TuxPdfResult<T> = Result<T, TuxPdfError>;

#[cfg(test)]
pub(crate) mod tests {
    use crate::document::PdfDocument;

    pub fn create_test_document(name: &str) -> crate::document::PdfDocument {
        let mut doc = crate::document::PdfDocument::new(name);
        doc.metadata.info.author = Some("tux-pdf tests".to_string());
        doc.metadata.info.creator = Some("tux-pdf tests".to_string());
        doc.metadata.info.creation_date =
            Some(time::OffsetDateTime::now_local().unwrap_or(time::OffsetDateTime::now_utc()));
        doc
    }

    pub fn save_pdf_doc(doc: PdfDocument, test_name: &str) -> anyhow::Result<()> {
        let save_location = destination_dir().join(format!("{}.pdf", test_name));
        let mut file = std::fs::File::create(save_location)?;
        let mut pdf = doc.write_to_lopdf_document()?;

        pdf.save_to(&mut file)?;

        Ok(())
    }

    include!("../tests/test_utils_external.rs");
}
