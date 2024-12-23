use std::borrow::Cow;
mod style;
use lopdf::Object;
pub use style::*;
mod group;
use crate::{
    document::{FontRef, PdfDocument, PdfResources},
    graphics::{OperationKeys, Point},
    TuxPdfError,
};
pub use group::*;
pub fn encode_text(
    resources: &PdfResources,
    text: &str,
    font: &FontRef,
) -> Result<Vec<u8>, TuxPdfError> {
    let text: Vec<_> = match font {
        FontRef::External(font_id) => {
            let font = resources
                .fonts
                .get_external_font(font_id)
                .ok_or(TuxPdfError::FontNotRegistered(font_id.clone()))?;
            text.chars()
                .filter_map(|char| font.get_glyph_id(char))
                .flat_map(|glyph_id| vec![(glyph_id >> 8) as u8, (glyph_id & 255) as u8])
                .collect()
        }
        FontRef::Builtin(builtin) => {
            if !resources.fonts.is_built_in_registered(builtin) {
                return Err(TuxPdfError::BuiltinFontNotRegistered(*builtin));
            }
            lopdf::Document::encode_text(&lopdf::Encoding::SimpleEncoding("WinAnsiEncoding"), text)
        }
    };
    Ok(text)
}
use super::{size::SimpleRenderSize, PdfOperation, PdfOperationType};

#[derive(Debug, PartialEq, Clone)]
pub struct Text<'text> {
    pub value: Cow<'text, str>,
    pub position: Point,
    pub style: TextStyle,
}

impl Text<'_> {
    pub fn new(value: impl Into<Cow<'static, str>>) -> Self {
        Self {
            value: value.into(),
            ..Default::default()
        }
    }
    pub fn with_style(value: impl Into<Cow<'static, str>>, style: TextStyle) -> Self {
        Self {
            value: value.into(),
            position: Point::default(),
            style,
        }
    }

    pub fn set_style(&mut self, style: TextStyle) {
        self.style = style;
    }
}
impl PdfOperationType for Text<'_> {
    fn write(
        &self,
        resources: &PdfResources,
        writer: &mut super::OperationWriter,
    ) -> Result<(), TuxPdfError> {
        writer.add_operation(OperationKeys::BeginText, vec![]);
        self.style.write(resources, writer)?;
        writer.add_operation(OperationKeys::TextPosition, self.position.into());
        let text = encode_text(resources, &self.value, &self.style.font_ref)?;
        writer.add_operation(
            OperationKeys::ShowText,
            vec![Object::String(text, lopdf::StringFormat::Hexadecimal)],
        );
        writer.add_operation(OperationKeys::EndText, vec![]);
        Ok(())
    }
}
impl From<Text<'static>> for PdfOperation {
    fn from(text: Text<'static>) -> Self {
        PdfOperation::WriteText(text)
    }
}
impl Default for Text<'_> {
    fn default() -> Self {
        Self {
            value: Cow::Borrowed(""),
            position: Point::default(),
            style: TextStyle::default(),
        }
    }
}
