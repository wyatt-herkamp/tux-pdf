use std::borrow::Cow;
mod modifiers;
use crate::{
    document::{BuiltinFont, FontRef, FontRenderSizeParams, PdfResources},
    graphics::{
        color::{Color, ColorWriter, HasColorParams},
        size::Size,
        OperationWriter, PdfObjectType,
    },
    units::Pt,
    utils::{IsEmpty, PartailOrFull, PartialStruct},
    TuxPdfError,
};
pub use modifiers::*;

use super::TextOperations;

#[derive(Debug, PartialEq, Clone)]
pub struct TextStyle {
    /// The size of the font
    ///
    /// Default is 12.0
    pub font_size: Pt,
    /// The font reference
    ///
    /// Defaults to Helvetica. Note. That you need to register the font with the pdf resources even if its built in.
    pub font_ref: FontRef,
    /// Defaults to previously set color or black
    pub fill_color: Option<Color>,
    pub outline_color: Option<Color>,
    /// Space between characters
    ///
    /// See [TextOperations::CharacterSpace] for more information
    pub character_spacing: Option<Pt>,
    /// Space between words
    ///
    /// See [TextOperations::WordSpace] for more information
    pub word_spacing: Option<Pt>,
    /// Uses to create superscript or subscript text
    ///
    /// See [TextOperations::TextRise] for more information
    pub text_rise: Option<Pt>,

    /// Space between lines
    ///
    /// ## Note
    /// This is not a pdf feature.
    ///
    /// Where when multiple lines in one text block are rendered, this is the space between them.
    pub line_spacing: Option<Pt>,
    /// Maximum width of text block
    ///
    /// ## Note
    ///
    /// This is not a pdf feature. This is a feature of this library where it attempts to wrap text.
    /// This is a work in progress and may not work as expected.
    ///
    /// ### What about max height?
    ///
    /// I do not see a use case for this currently. If you have one please open an issue.
    ///
    /// But the main use case for this is to put text in a a table cell.
    pub max_width: Option<Pt>,
    /// Minimum width of text block
    pub min_width: Option<Pt>,
}

impl HasColorParams for TextStyle {
    fn set_fill_color(&mut self, color: Color) {
        self.fill_color = Some(color);
    }

    fn set_outline_color(&mut self, color: Color) {
        self.outline_color = Some(color);
    }

    fn take_fill_color(&mut self) -> Option<Color> {
        self.fill_color.take()
    }

    fn take_outline_color(&mut self) -> Option<Color> {
        self.outline_color.take()
    }
}
impl PdfObjectType for TextStyle {
    fn write(
        self,
        resources: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), TuxPdfError> {
        writer.add_operation(
            TextOperations::TextFont,
            vec![self.font_ref.into(), self.font_size.into()],
        );
        if let Some(text_rise) = self.text_rise {
            writer.add_operation(TextOperations::TextRise, vec![text_rise.into()]);
        }
        if let Some(character_spacing) = self.character_spacing {
            writer.add_operation(
                TextOperations::CharacterSpace,
                vec![character_spacing.into()],
            );
        }
        if let Some(word_spacing) = self.word_spacing {
            writer.add_operation(TextOperations::WordSpace, vec![word_spacing.into()]);
        }
        let color_writer = ColorWriter {
            outline_color: self.outline_color.map(Cow::Owned),
            fill_color: self.fill_color.map(Cow::Owned),
        };
        color_writer.write(resources, writer)?;
        Ok(())
    }
}

impl FontRenderSizeParams for TextStyle {
    fn font_size(&self) -> Pt {
        self.font_size
    }

    fn character_spacing(&self) -> Option<Pt> {
        self.character_spacing
    }

    fn word_spacing(&self) -> Option<Pt> {
        self.word_spacing
    }

    fn text_rise(&self) -> Option<Pt> {
        self.text_rise
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: Pt(12.0),
            font_ref: FontRef::Builtin(BuiltinFont::Helvetica),
            fill_color: None,
            outline_color: None,
            word_spacing: None,
            line_spacing: None,
            max_width: None,
            character_spacing: None,
            text_rise: None,
            min_width: None,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct PartialTextStyle {
    pub font_size: Option<Pt>,
    pub font_ref: Option<FontRef>,
    /// Defaults to previously set color or black
    pub fill_color: Option<Color>,
    pub outline_color: Option<Color>,
    pub word_spacing: Option<Pt>,

    pub max_width: Option<Pt>,
}
impl IsEmpty for PartialTextStyle {
    fn is_empty(&self) -> bool {
        self.font_size.is_none()
            && self.font_ref.is_none()
            && self.fill_color.is_none()
            && self.outline_color.is_none()
            && self.word_spacing.is_none()
            && self.max_width.is_none()
    }
}
pub type PartialOrFullTextStyle = PartailOrFull<PartialTextStyle>;
impl PartialStruct for PartialTextStyle {
    type FullStruct = TextStyle;

    fn merge_with_full<'full>(
        &self,
        full: &'full Self::FullStruct,
    ) -> Cow<'full, Self::FullStruct> {
        if self.is_empty() {
            return Cow::Borrowed(full);
        }
        let mut new = full.clone();
        if let Some(font_size) = self.font_size {
            new.font_size = font_size;
        }
        if let Some(font_ref) = self.font_ref.clone() {
            new.font_ref = font_ref;
        }
        if let Some(color) = self.fill_color.clone() {
            new.fill_color = Some(color);
        }
        if let Some(outline_color) = self.outline_color.clone() {
            new.outline_color = Some(outline_color);
        }
        if let Some(word_spacing) = self.word_spacing {
            new.word_spacing = Some(word_spacing);
        }
        if let Some(max_width) = self.max_width {
            new.max_width = Some(max_width);
        }
        Cow::Owned(new)
    }
}
