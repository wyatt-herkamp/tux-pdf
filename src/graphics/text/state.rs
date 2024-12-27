use crate::{
    document::{FontRef, FontRenderSizeParams, InternalFontTypes, PdfResources},
    graphics::OperationWriter,
    units::Pt,
    utils::IsEmpty,
    TuxPdfError,
};

use super::{TextOperations, TextStyle};
/// Used to store the current state of the text block
///
/// This is only used for managing text size
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TextBlockState<'resources> {
    pub(crate) resources: &'resources PdfResources,
    pub(crate) font: FontRef,
    pub(crate) font_size: Pt,
    pub(crate) font_type: InternalFontTypes<'resources>,
    pub(crate) word_spacing: Option<Pt>,
    pub(crate) character_spacing: Option<Pt>,
    pub(crate) text_rise: Option<Pt>,
}
impl<'resources> TextBlockState<'resources> {
    pub fn new(
        resources: &'resources PdfResources,
        styles: &TextStyle,
    ) -> Result<Self, TuxPdfError> {
        let font_type = resources
            .fonts
            .internal_font_type(&styles.font_ref)
            .ok_or_else(|| TuxPdfError::from(styles.font_ref.clone()))?;
        Ok(Self {
            resources,
            font: styles.font_ref.clone(),
            font_size: styles.font_size,
            font_type,
            word_spacing: styles.word_spacing,
            character_spacing: styles.character_spacing,
            text_rise: styles.text_rise,
        })
    }
    pub fn create_updating<'state>(&'state self) -> UpdatingTextBlockState<'state, 'resources> {
        UpdatingTextBlockState {
            original: self,
            font: None,
            font_size: None,
            word_spacing: None,
            character_spacing: None,
            text_rise: None,
        }
    }
}

impl FontRenderSizeParams for TextBlockState<'_> {
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
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UpdatingTextBlockState<'state, 'resources> {
    pub(crate) original: &'state TextBlockState<'resources>,
    pub(crate) font: Option<FontRef>,
    pub(crate) font_size: Option<Pt>,
    pub(crate) word_spacing: Option<Pt>,
    pub(crate) character_spacing: Option<Pt>,
    pub(crate) text_rise: Option<Pt>,
}
impl IsEmpty for UpdatingTextBlockState<'_, '_> {
    fn is_empty(&self) -> bool {
        self.font.is_none()
            && self.font_size.is_none()
            && self.word_spacing.is_none()
            && self.character_spacing.is_none()
            && self.text_rise.is_none()
    }
}
impl<'resources> UpdatingTextBlockState<'_, 'resources> {
    pub fn build(
        self,
        writer: Option<&mut OperationWriter>,
    ) -> Result<Option<TextBlockState<'resources>>, TuxPdfError> {
        // If no changes were made, return the original state
        if self.is_empty() {
            return Ok(None);
        }
        let font_type = if let Some(font) = self.font.as_ref() {
            self.original
                .resources
                .fonts
                .internal_font_type(font)
                .ok_or_else(|| TuxPdfError::from(font.clone()))?
        } else {
            self.original.font_type
        };
        if let Some(writer) = writer {
            if self.font.is_some() || self.font_size.is_some() {
                let font = self.font.as_ref().unwrap_or(&self.original.font);
                let font_size = self.font_size.unwrap_or(self.original.font_size);
                writer.add_operation(
                    TextOperations::TextFont,
                    vec![font.clone().into(), font_size.into()],
                );
            }
        }
        let result = TextBlockState {
            resources: self.original.resources,
            font: self.font.unwrap_or_else(|| self.original.font.clone()),
            font_size: self.font_size.unwrap_or(self.original.font_size),
            font_type,
            word_spacing: self.word_spacing.or(self.original.word_spacing),
            character_spacing: self.character_spacing.or(self.original.character_spacing),
            text_rise: self.text_rise.or(self.original.text_rise),
        };
        Ok(Some(result))
    }
}
