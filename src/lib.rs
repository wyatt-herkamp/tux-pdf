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
    use std::{path::PathBuf, sync::Once};
    use tracing::{error, info, level_filters::LevelFilter};
    use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt, Layer};
    /// Returns the path to the fonts directory in the tests directory
    pub fn test_fonts_directory() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fonts")
    }
    pub fn does_end_with_ttf(path: &std::path::Path) -> bool {
        path.extension().map_or(false, |ext| ext == "ttf")
    }
    pub fn init_logger() {
        static ONCE: Once = std::sync::Once::new();
        ONCE.call_once(|| {
            let stdout_log = tracing_subscriber::fmt::layer().pretty().without_time();
            tracing_subscriber::registry()
                .with(
                    stdout_log.with_filter(
                        filter::Targets::new().with_target("tux_pdf", LevelFilter::TRACE),
                    ),
                )
                .init();
        });
        info!("Logger initialized");
        error!("This is an error message");
    }
}
