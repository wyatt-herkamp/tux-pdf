use lopdf::Dictionary;

use crate::{
    document::types::{BuiltinFontSubType, FontEncoding, FontObject, PdfDirectoryType},
    graphics::size::Size,
    units::Pt,
};

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
impl BuiltinFont {
    pub fn calculate_size_of_text(&self, text: &str, font_size: Pt) -> Size {
        Size::new(Pt(0f32), Pt(0f32))
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
