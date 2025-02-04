use crate::types::{Name, NullOrObject};

use super::Dictionary;
#[derive(Debug, Clone, PartialEq)]
pub enum PdfStreamFilter {
    Ascii85Decode,
    /// Requires the flate2 feature to be enabled
    FlateDecode,
    /// Requires the weezl feature to be enabled
    LZWDecode,
    Other(Name),
}
impl From<Name> for PdfStreamFilter {
    fn from(name: Name) -> Self {
        match name.as_slice() {
            b"ASCII85Decode" => Self::Ascii85Decode,
            b"FlateDecode" => Self::FlateDecode,
            b"LZWDecode" => Self::LZWDecode,
            _ => Self::Other(name),
        }
    }
}
/// See 7.3.8.2 Table 5
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PdfStreamDictionary {
    pub length: i64,
    /// The filters that are applied to the stream
    ///
    /// ## Node
    /// Does not have to be an array if only one filter is applied
    pub filter: Vec<PdfStreamFilter>,
    /// If you have 1 filter then the decode params may possibly be omitted
    ///
    ///
    /// If you have multiple and one of them requires decode params you must have an equal amount of decode_params to filters
    ///
    /// But the array may contain nulls
    ///
    /// ## Node
    ///
    /// Does not have to be an array if only one filter is applied
    pub decode_params: Vec<NullOrObject<Dictionary>>,
}
