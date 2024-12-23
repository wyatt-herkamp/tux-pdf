use std::borrow::Cow;

use crate::{
    document::{BuiltinFont, FontRef, PdfResources},
    graphics::{
        color::{Color, ColorWriter},
        OperationKeys, OperationWriter, PdfOperationType,
    },
    units::Pt,
    utils::{IsEmpty, PartailOrFull, PartialStruct},
    TuxPdfError,
};

#[derive(Debug, PartialEq, Clone)]
pub struct TextStyle {
    pub font_size: Pt,
    pub font_ref: FontRef,
    /// Defaults to previously set color or black
    pub fill_color: Option<Color>,
    pub outline_color: Option<Color>,
    pub word_spacing: Option<Pt>,

    pub max_width: Option<Pt>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: Pt(12.0),
            font_ref: FontRef::internal_default(),
            fill_color: None,
            outline_color: None,
            word_spacing: None,
            max_width: None,
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

impl PdfOperationType for TextStyle {
    fn write(
        &self,
        resources: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), TuxPdfError> {
        writer.add_operation(
            OperationKeys::TextFont,
            vec![self.font_ref.clone().into(), self.font_size.into()],
        );

        let color_writer = ColorWriter {
            outline_color: self.outline_color.as_ref(),
            fill_color: self.fill_color.as_ref(),
        };
        color_writer.write(resources, writer)?;
        Ok(())
    }
}
