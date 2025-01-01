//! CID Fonts
//!
//! Section 9.7.4

use std::borrow::Cow;

use derive_builder::Builder;
use either::Either;
use tux_pdf_low::types::{Dictionary, Object, ObjectId, Stream};

use crate::document::types::PdfDirectoryType;

use super::FontSubType;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Builder)]
pub struct CIDSystemInfo<'font> {
    pub registry: Cow<'font, str>,
    pub ordering: Cow<'font, str>,
    pub supplement: i64,
}
impl PdfDirectoryType for CIDSystemInfo<'_> {
    fn dictionary_type_key() -> &'static str {
        "CIDSystemInfo"
    }

    fn into_dictionary(self) -> Dictionary {
        let mut dict = Dictionary::new();
        let registry = Object::string_literal_owned(self.registry.as_bytes().to_vec());
        let ordering = Object::string_literal_owned(self.ordering.as_bytes().to_vec());
        dict.set("Registry", registry);
        dict.set("Ordering", ordering);
        dict.set("Supplement", self.supplement);
        dict
    }
}
#[derive(Debug, Clone)]
pub struct CidFontType2<'font> {
    pub cid_system_info: CIDSystemInfo<'font>,
    pub font_descriptor: ObjectId,
    pub dw: Option<i64>,
    pub w: Option<Vec<Object>>,
    pub dw2: Option<Vec<Object>>,
    pub w2: Option<Vec<Object>>,
    pub cid_to_gid_map: Option<Either<Stream, Cow<'font, str>>>,
}
impl FontSubType for CidFontType2<'_> {
    fn sub_type(&self) -> &str {
        "CIDFontType2"
    }
    fn write_to_dictionary(&self, cid_font_dict: &mut Dictionary) {
        cid_font_dict.set(
            "CIDSystemInfo",
            self.cid_system_info.clone().into_dictionary(),
        );
        cid_font_dict.set("FontDescriptor", self.font_descriptor);

        if let Some(dw) = self.dw {
            cid_font_dict.set("DW", dw);
        }
        if let Some(w) = &self.w {
            cid_font_dict.set("W", w.clone());
        }
        if let Some(dw2) = &self.dw2 {
            cid_font_dict.set("DW2", dw2.clone());
        }
        if let Some(w2) = &self.w2 {
            cid_font_dict.set("W2", w2.clone());
        }
        if let Some(cid_to_gid_map) = &self.cid_to_gid_map {
            match cid_to_gid_map {
                Either::Left(stream) => {
                    cid_font_dict.set("CIDToGIDMap", Object::Stream(stream.clone()))
                }
                Either::Right(name) => {
                    cid_font_dict.set("CIDToGIDMap", Object::name(name.as_bytes().to_vec()))
                }
            }
        }
    }
}
