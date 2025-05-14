use std::{mem, ops::Deref};

use crate::{
    TuxPdfError,
    document::{FontRef, FontType, PdfDocument},
    graphics::{
        OperationKeys,
        size::{RenderSize, Size},
        state_from_modifiers,
    },
    units::Pt,
};

use tracing::debug;
use tux_pdf_low::types::Object;

use super::{
    OperationWriter, TextBlockState, TextModifier, TextOperations, TextStyle, write_modifiers,
};

/// A text item is a string with a list of modifiers
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextItem {
    pub text: String,
    pub modifiers: Vec<TextModifier>,
}
impl From<String> for TextItem {
    fn from(text: String) -> Self {
        Self {
            text,
            ..Default::default()
        }
    }
}

impl TextItem {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            modifiers: Vec::new(),
        }
    }
    pub fn with_font_size(mut self, size: Pt) -> Self {
        self.modifiers.push(TextModifier::FontSize(size));
        self
    }
    pub fn with_font(mut self, font: FontRef) -> Self {
        self.modifiers.push(TextModifier::Font(font));
        self
    }

    pub fn with_text_rise(mut self, rise: Pt) -> Self {
        self.modifiers.push(TextModifier::TextRise(rise));
        self
    }
    pub fn with_character_spacing(mut self, spacing: Pt) -> Self {
        self.modifiers.push(TextModifier::CharacterSpacing(spacing));
        self
    }
    pub fn with_word_spacing(mut self, spacing: Pt) -> Self {
        self.modifiers.push(TextModifier::WordSpacing(spacing));
        self
    }
    /// Splits the text into two items once the available width is reached
    ///
    /// Returns the remaining text if the text was split
    fn cut_off_at_max(
        &mut self,
        availalble_width: Pt,
        current_state: &TextBlockState,
    ) -> Result<(Pt, Option<Self>), TuxPdfError> {
        let state = state_from_modifiers(&self.modifiers, current_state)?;

        let text_size = state
            .font_type
            .calculate_size_of_text(&self.text, state.as_ref());
        if text_size.width < availalble_width {
            return Ok((text_size.width, None));
        }

        let mut ending_index: Option<(usize, usize)> = Option::None;
        let mut width = Pt::default();
        let mut last_good_break = None;
        for (index, c) in self.text.chars().enumerate() {
            let char_size = current_state
                .font_type
                .size_of_char(c, state.as_ref())
                .unwrap_or_default();
            if width + char_size.width > availalble_width {
                if let Some(last_good_break) = last_good_break {
                    ending_index = Some((last_good_break, last_good_break + 1));
                    break;
                } else {
                    ending_index = Some((index, index));
                    break;
                }
            }
            if c.is_whitespace() {
                last_good_break = Some(index);
            }
            width += char_size.width;
        }
        let (end, start) = ending_index.unwrap_or((self.text.len(), self.text.len()));

        let remaining_text = self.text[start..].to_string();
        debug!("Remaining Text: {:?}", remaining_text);
        self.text = self.text[..end].to_string();
        let new_line = Some(Self {
            text: remaining_text,
            modifiers: self.modifiers.clone(),
        });
        Ok((width, new_line))
    }

    pub(crate) fn calculate_size_of_text(
        &self,
        current_state: &TextBlockState,
    ) -> Result<Size, TuxPdfError> {
        let state = state_from_modifiers(&self.modifiers, current_state)?;
        Ok(state
            .font_type
            .calculate_size_of_text(&self.text, state.as_ref()))
    }
    fn write(
        self,
        current_state: &TextBlockState,
        writer: &mut OperationWriter,
    ) -> Result<Size, TuxPdfError> {
        let state = write_modifiers(self.modifiers, current_state, writer)?;

        debug!(?state, "Text State for Text Item");
        let text_size = state
            .font_type
            .calculate_size_of_text(&self.text, state.as_ref());
        let text = state.font_type.encode_text(&self.text);

        writer.add_operation(
            TextOperations::ShowText,
            vec![Object::String(tux_pdf_low::types::PdfString::Hexadecimal(
                text,
            ))],
        );
        Ok(text_size)
    }
}
#[derive(Debug, Clone, PartialEq)]
enum LineMaxWidth {
    LeftoverSpace(Pt),
    SplitLines,
}
/// A text line is a list of text items
///
/// Shared modifiers are applied to all items in the line
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextLine {
    pub items: Vec<TextItem>,
    pub modifiers: Vec<TextModifier>,
}
impl TextLine {
    /// Adds an item to the line using a builder pattern
    ///
    /// If you want add an item without that pattern use `items.push(item)`
    pub fn add_item(mut self, item: impl Into<TextItem>) -> Self {
        self.items.push(item.into());
        self
    }
    pub(super) fn write(
        self,
        current_state: &TextBlockState,
        writer: &mut OperationWriter,
    ) -> Result<Size, TuxPdfError> {
        let mut line_size: Size = Size::default();
        write_modifiers(self.modifiers, current_state, writer)?;

        for item in self.items {
            let restore = if !item.modifiers.is_empty() {
                writer.push_empty_op(OperationKeys::SaveGraphicsState);
                true
            } else {
                false
            };
            let item_size = item.write(current_state, writer)?;
            if restore {
                writer.push_empty_op(OperationKeys::RestoreGraphicsState);
            }
            line_size.width += item_size.width;
            line_size.height = line_size.height.max(item_size.height);
        }

        Ok(line_size)
    }
    fn calculate_size_of_text(&self, current_state: &TextBlockState) -> Result<Size, TuxPdfError> {
        let state = state_from_modifiers(&self.modifiers, current_state)?;
        let mut line_size: Size = Size::default();
        for item in &self.items {
            let item_size = item.calculate_size_of_text(state.as_ref())?;
            line_size.width += item_size.width;
            line_size.height = line_size.height.max(item_size.height);
        }
        Ok(line_size)
    }

    fn apply_max_width(
        mut self,
        max_width: Pt,
        current_state: &TextBlockState,
        lines: &mut Vec<TextLine>,
    ) -> Result<LineMaxWidth, TuxPdfError> {
        let mut item_iterator = mem::take(&mut self.items).into_iter();
        // Read Line Items until the max width is reached
        let mut item = item_iterator.next();
        let mut extra_items = Vec::new();

        let mut current_max_width = max_width;
        while let Some(mut current_item) = item {
            let (width, new_item) =
                current_item.cut_off_at_max(current_max_width, current_state)?;
            current_max_width -= width;
            if let Some(new_item) = new_item {
                self.items.push(current_item);
                extra_items.push(new_item);
                break;
            } else {
                self.items.push(current_item);
                item = item_iterator.next();
            }
        }
        if extra_items.is_empty() {
            lines.push(self);
            Ok(LineMaxWidth::LeftoverSpace(current_max_width))
        } else {
            let modifiers = self.modifiers.clone();
            lines.push(self);
            extra_items.extend(item_iterator);
            let new_line = TextLine {
                items: extra_items,
                modifiers,
            };
            new_line.apply_max_width(max_width, current_state, lines)?;
            Ok(LineMaxWidth::SplitLines)
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextBlockContent(pub Vec<TextLine>);
impl TextBlockContent {
    /// Adds a line to the block using a builder pattern
    ///
    /// If you want add a line without that pattern use `lines.push(line)`
    pub fn add_line(mut self, line: impl Into<TextLine>) -> Self {
        self.0.push(line.into());
        self
    }

    pub(super) fn apply_max_width_inner(
        &mut self,
        max_width: Pt,
        current_state: &TextBlockState,
    ) -> Result<(), TuxPdfError> {
        let old_lines = mem::take(&mut self.0);

        for line in old_lines {
            let current_max_width = max_width;
            let result = line.apply_max_width(current_max_width, current_state, &mut self.0)?;

            debug!(?result, "Line Max Width Result");
        }
        Ok(())
    }
    pub fn apply_max_width(
        &mut self,
        max_width: Pt,
        document: &PdfDocument,
        style: &TextStyle,
    ) -> Result<(), TuxPdfError> {
        let state = TextBlockState::new(&document.resources, style)?;
        self.apply_max_width_inner(max_width, &state)?;
        Ok(())
    }
}
impl RenderSize for TextBlockContent {
    type Settings = TextStyle;
    type Error = TuxPdfError;

    fn render_size(
        &self,
        document: &crate::document::PdfDocument,
        settings: &TextStyle,
    ) -> Result<Size, TuxPdfError> {
        let state = TextBlockState::new(&document.resources, settings)?;

        let mut size: Size = Size::default();

        for line in &self.0 {
            let line_size = line.calculate_size_of_text(&state)?;
            size.width = size.width.max(line_size.width);
            size.height += line_size.height;
        }

        Ok(size)
    }
}
impl Deref for TextBlockContent {
    type Target = Vec<TextLine>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<String> for TextBlockContent {
    fn from(text: String) -> Self {
        let lines = str_to_lines(&text);
        Self(lines)
    }
}
impl From<&str> for TextBlockContent {
    fn from(text: &str) -> Self {
        let lines = str_to_lines(text);
        Self(lines)
    }
}
impl From<Vec<TextLine>> for TextBlockContent {
    fn from(lines: Vec<TextLine>) -> Self {
        Self(lines)
    }
}
impl From<Vec<String>> for TextBlockContent {
    fn from(lines: Vec<String>) -> Self {
        let lines = lines
            .into_iter()
            .flat_map(|line| str_to_lines(&line))
            .collect();
        Self(lines)
    }
}
impl From<String> for TextLine {
    fn from(text: String) -> Self {
        Self {
            items: vec![TextItem::new(text)],
            modifiers: Vec::new(),
        }
    }
}
impl From<TextLine> for TextBlockContent {
    fn from(line: TextLine) -> Self {
        Self(vec![line])
    }
}
impl From<&str> for TextLine {
    fn from(text: &str) -> Self {
        Self {
            items: vec![TextItem::new(text)],
            modifiers: Vec::new(),
        }
    }
}
fn str_to_lines(text: &str) -> Vec<TextLine> {
    text.lines()
        .map(|line| TextLine {
            items: vec![TextItem::new(line)],
            modifiers: Vec::new(),
        })
        .collect()
}
