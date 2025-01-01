use tux_pdf_low::types::Dictionary;

use crate::{
    document::types::{BuiltinFontSubType, FontEncoding, FontObject, PdfDirectoryType},
    graphics::size::Size,
    units::{Pt, UnitType},
};

use super::{FontRenderSizeParams, FontType};
///pub static WIN_ANSI_ENCODING: Encoding<'static> =
///    lopdf::Encoding::SimpleEncoding("WinAnsiEncoding");

/// The 14 built-in fonts per the PDF specification.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltinFont {
    TimesRoman,
    TimesBold,
    TimesItalic,
    TimesBoldItalic,
    Helvetica,
    HelveticaBold,
    HelveticaOblique,
    HelveticaBoldOblique,
    Courier,
    CourierOblique,
    CourierBold,
    CourierBoldOblique,
    Symbol,
    ZapfDingbats,
}
impl FontType for BuiltinFont {
    fn calculate_size_of_text<P: FontRenderSizeParams>(&self, text: &str, params: &P) -> Size {
        // TODO figure out how to calculate the size of text for built-in fonts
        Size {
            width: (params.font_size() * Pt::from(text.len())) / 2f32.pt(),
            height: params.font_size(),
        }
    }

    fn size_of_char<P: FontRenderSizeParams>(&self, _: char, params: &P) -> Option<Size> {
        Some(Size {
            width: (params.font_size()) / 2f32.pt(),
            height: params.font_size(),
        })
    }
    fn encode_text(&self, text: &str) -> Vec<u8> {
        text.as_bytes().to_vec()
    }
}
impl From<BuiltinFont> for Dictionary {
    fn from(value: BuiltinFont) -> Self {
        FontObject {
            sub_type: BuiltinFontSubType,
            encoding: Some(FontEncoding::WinAnsiEncoding),
            base_font: value.name().into(),
        }
        .into_dictionary()
    }
}
macro_rules! builtin_font {
    (
        $(
            $variant:ident => $name:literal => $id:literal
        ),*
    ) => {
        impl BuiltinFont {
            /// The name of the font as it is used in PDF documents.
            pub fn name(&self) -> &'static str {
                match self {
                    $(BuiltinFont::$variant => $name,)*
                }
            }
            /// We use a dedicated identifier for each built-in font.
            ///
            /// For example, the Times-Roman font has the identifier `F1`.
            #[doc(hidden)]
            pub fn dedicated_font_id(&self) -> &'static str {
                match self {
                    $(BuiltinFont::$variant => $id,)*
                }
            }
        }
        impl TryFrom<&str> for BuiltinFont {
            type Error = ();
            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value {
                    $($name => Ok(BuiltinFont::$variant),)*
                    _ => Err(()),
                }
            }
        }
        impl std::fmt::Display for BuiltinFont {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.name())
            }
        }

    };
}
builtin_font!(
    TimesRoman => "Times-Roman" => "F1",
    TimesBold => "Times-Bold" => "F2",
    TimesItalic => "Times-Italic" => "F3",
    TimesBoldItalic => "Times-BoldItalic" => "F4",
    Helvetica => "Helvetica" => "F5",
    HelveticaBold => "Helvetica-Bold" => "F6",
    HelveticaOblique => "Helvetica-Oblique" => "F7",
    HelveticaBoldOblique => "Helvetica-BoldOblique" => "F8",
    Courier => "Courier" => "F9",
    CourierOblique => "Courier-Oblique" => "F10",
    CourierBold => "Courier-Bold" => "F11",
    CourierBoldOblique => "Courier-BoldOblique" => "F12",
    Symbol => "Symbol" => "F13",
    ZapfDingbats => "ZapfDingbats" => "F14"
);
