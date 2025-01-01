use std::borrow::Cow;
mod cid_fonts;
mod font_descriptor;
pub use cid_fonts::*;
use derive_builder::Builder;
pub use font_descriptor::*;
use strum::{AsRefStr, Display, EnumString};
use tux_pdf_low::{
    dictionary,
    types::{Dictionary, Name, Object, ObjectId, Stream},
};

use super::PdfDirectoryType;
/// A font subtype
///
/// Use [BuiltinFontSubType] for built-in fonts
///
/// Use [Type1Font] for Type1 fonts
pub trait FontSubType {
    /// The font's subtype
    fn sub_type(&self) -> &str;

    fn write_to_dictionary(&self, dict: &mut Dictionary);
}
/// Yes they share the same sub type as [Type1Font]
/// But basically most of the parameters all have a claus saying except for the 14 built-in fonts
#[derive(Debug, Clone, Default)]
pub struct BuiltinFontSubType;
/// Used for Built-in fonts
impl FontSubType for BuiltinFontSubType {
    fn sub_type(&self) -> &str {
        "Type1"
    }
    /// Does nothing
    #[inline(always)]
    fn write_to_dictionary(&self, _dict: &mut Dictionary) {}
}
/// Type1 font that is not built-in
///
/// Section 9.6.2
pub struct Type1Font {
    pub first_char: i64,
    pub last_char: i64,
    pub widths: Vec<Object>,
    pub font_descriptor: ObjectId,
    pub to_unicode: Option<Stream>,
}
impl FontSubType for Type1Font {
    fn sub_type(&self) -> &str {
        "Type1"
    }
    fn write_to_dictionary(&self, dict: &mut Dictionary) {
        dict.set("FirstChar", self.first_char);
        dict.set("LastChar", self.last_char);
        dict.set("Widths", self.widths.clone());
        dict.set("FontDescriptor", self.font_descriptor);
        if let Some(to_unicode) = &self.to_unicode {
            dict.set("ToUnicode", Object::Stream(to_unicode.clone()));
        }
    }
}
pub struct Type0Font {
    pub descendant_fonts: Vec<Dictionary>,
    pub to_unicode: Option<ObjectId>,
}
impl FontSubType for Type0Font {
    fn sub_type(&self) -> &str {
        "Type0"
    }
    fn write_to_dictionary(&self, dict: &mut Dictionary) {
        let descendant_fonts = Object::Array(
            self.descendant_fonts
                .clone()
                .into_iter()
                .map(Object::Dictionary)
                .collect::<Vec<_>>(),
        );
        dict.set("DescendantFonts", descendant_fonts);
        if let Some(to_unicode) = &self.to_unicode {
            dict.set("ToUnicode", Object::Reference(*to_unicode));
        }
    }
}
#[derive(Debug, PartialEq, Clone, Eq, PartialOrd, Ord, EnumString, AsRefStr, Display)]
pub enum FontEncoding {
    #[strum(serialize = "WinAnsiEncoding")]
    WinAnsiEncoding,
    /// The horizontal identify mapping for 2-byte CIDs
    ///
    /// May e used with CID fonts using any Registry, Ordering, and Supplement values
    #[strum(serialize = "Identity-H")]
    IdentityH,
    /// The vertical identify mapping for 2-byte CIDs
    #[strum(serialize = "Identity-V")]
    IdentityV,
}
#[derive(Debug, Builder)]
pub struct FontObject<'font, SubType: FontSubType> {
    /// The font's subtype
    ///
    /// See [FontSubType]
    pub sub_type: SubType,
    /// The font's encoding
    pub encoding: Option<FontEncoding>,
    /// The font's base font
    pub base_font: Cow<'font, str>,
}
impl<SubType: FontSubType> PdfDirectoryType for FontObject<'_, SubType> {
    fn dictionary_type_key() -> &'static str {
        "Font"
    }
    fn into_dictionary(self) -> Dictionary {
        let base_font = Object::name(self.base_font.as_ref());
        let encoding = self
            .encoding
            .map(|encoding| Object::name(encoding.as_ref().as_bytes().to_vec()));
        let sub_type = Object::name(self.sub_type.sub_type());
        let mut dictionary = dictionary! {
            "Type" => Name::from(Self::dictionary_type_key()),
            "Subtype" => sub_type,
            "BaseFont" => base_font
        };
        if let Some(encoding) = encoding {
            dictionary.set("Encoding", encoding);
        }
        self.sub_type.write_to_dictionary(&mut dictionary);
        dictionary
    }
}
