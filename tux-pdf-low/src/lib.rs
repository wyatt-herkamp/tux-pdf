use thiserror::Error;
pub mod content;
pub mod document;
pub mod types;
pub mod utils;
#[derive(Debug, Error)]
pub enum LowTuxPdfError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Invalid dictionary type for dictionary: {actual}, expected: {expected}")]
    InvalidDictionaryType {
        actual: &'static str,
        expected: &'static str,
    },
    #[error("Missing dictionary key: {0}")]
    MissingDictionaryKey(String),
    #[error("Invalid Type for Dictionary Value")]
    InvalidDictionaryValue {
        actual: &'static str,
        expected: &'static str,
    },
}
#[test]
pub fn test() {}
