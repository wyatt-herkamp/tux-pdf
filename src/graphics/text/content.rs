use std::{mem, ops::Deref};

use crate::{
    TuxPdfError,
    document::{
        ExternalLoadedFont, FontRef, FontType, GlyphMetrics, InternalFontTypes, PdfDocument,
    },
    graphics::{
        OperationKeys, PdfObjectType, PdfPosition,
        primitives::ctm::CurTransMat,
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

/// Tracks the absolute cursor position during text rendering.
///
/// Used when emoji images need to be placed inline, requiring knowledge
/// of the current absolute position to break out of BT/ET and re-enter.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TextCursor {
    pub x: Pt,
    pub y: Pt,
    /// The origin position from the initial BT/Td. After breaking out of BT for emoji,
    /// we re-enter BT at this origin so that subsequent relative Td operations
    /// (e.g., line spacing) remain correct.
    pub origin_x: Pt,
    #[allow(dead_code)]
    pub origin_y: Pt,
}

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
        cursor: &mut TextCursor,
    ) -> Result<Size, TuxPdfError> {
        let Self { text, modifiers } = self;
        let state = write_modifiers(modifiers, current_state, writer)?;

        debug!(?state, "Text State for Text Item");

        // Check if this item uses a color emoji font with cached glyphs
        if let InternalFontTypes::External(parsed_font) = &state.font_type
            && parsed_font.has_color_glyphs
            && !state.resources.emoji_cache.is_empty()
        {
            return Self::write_emoji_images(&text, &state, parsed_font, writer, cursor);
        }

        let text_size = state
            .font_type
            .calculate_size_of_text(&text, state.as_ref());
        let text = state.font_type.encode_text(&text);

        writer.add_operation(
            TextOperations::ShowText,
            vec![Object::String(tux_pdf_low::types::PdfString::Hexadecimal(
                text,
            ))],
        );

        cursor.x += text_size.width;
        Ok(text_size)
    }

    /// Renders emoji characters as inline images by breaking out of BT/ET.
    fn write_emoji_images(
        text: &str,
        state: &TextBlockState<'_>,
        parsed_font: &crate::document::ParsedFont,
        writer: &mut OperationWriter,
        cursor: &mut TextCursor,
    ) -> Result<Size, TuxPdfError> {
        let emoji_cache = &state.resources.emoji_cache;
        let font_name = &parsed_font.font_name;
        let font_size = state.font_size;
        let units_per_em = parsed_font.font.units_per_em();

        let mut total_width = Pt::default();
        let mut max_height = Pt::default();

        for c in text.chars() {
            let Some(glyph_id) = parsed_font.font.glyph_id(c) else {
                continue;
            };

            // Get glyph metrics for positioning
            let glyph_metrics: Option<GlyphMetrics> = parsed_font.font.glyph_metrics(glyph_id);
            let (glyph_width, glyph_height) = glyph_metrics
                .map(|m| m.glyph_size_in_points(units_per_em, font_size))
                .unwrap_or((font_size, font_size));

            if let Some(xobject_id) = emoji_cache.get(font_name, glyph_id) {
                // End current text object
                writer.push_empty_op(TextOperations::EndText);

                // Draw the emoji image
                writer.save_graphics_state();

                // PDF text cursor.y is the baseline. The rasterized emoji image covers
                // the full ascender-to-descender range (viewBox "0 -asc em asc-desc").
                // Image width = units_per_em, image height = ascender - descender.
                // Scale: width maps to font_size, height maps to the full asc-desc range.
                let ascender = parsed_font.font.ascender() as f32;
                let descender = parsed_font.font.descender() as f32;
                let scale_factor = font_size.0 / units_per_em as f32;

                let emoji_width = font_size; // units_per_em * scale_factor = font_size
                let emoji_height = Pt((ascender - descender) * scale_factor);
                let y_pos = cursor.y + Pt(descender * scale_factor);

                let transforms = vec![
                    CurTransMat::Scale(emoji_width, emoji_height),
                    CurTransMat::Position(PdfPosition {
                        x: cursor.x,
                        y: y_pos,
                    }),
                ];
                transforms.write(state.resources, writer)?;

                writer.add_operation(OperationKeys::PaintXObject, vec![xobject_id.clone().into()]);
                writer.restore_graphics_state();

                // Advance cursor by the glyph's actual advance width (may be wider than
                // the em-square, e.g. NotoColorEmoji uses ~1275 units vs 1024 em)
                cursor.x += glyph_width;
                total_width += glyph_width;

                // Re-enter text mode with absolute positioning via Tm.
                // Tm sets both the text matrix AND text line matrix, so subsequent
                // relative Td (e.g., line spacing) will be relative to this position.
                // We set the line matrix to (origin_x, cursor.y) so that a later
                // Td(0, lineHeight) correctly moves to the next line at the left margin,
                // then use a relative Td to advance X to the current cursor position.
                writer.push_empty_op(TextOperations::BeginText);
                writer.add_operation(
                    TextOperations::SetTextMatrix,
                    vec![
                        Object::Real(1.0),
                        Object::Real(0.0),
                        Object::Real(0.0),
                        Object::Real(1.0),
                        cursor.origin_x.into(),
                        cursor.y.into(),
                    ],
                );
                // Advance from line start to current cursor X
                writer.add_operation(
                    TextOperations::TextPosition,
                    PdfPosition {
                        x: cursor.x - cursor.origin_x,
                        y: Pt::default(),
                    }
                    .into(),
                );
                // Re-set font (will be overridden by Q if inside a modifier block)
                writer.add_operation(
                    TextOperations::TextFont,
                    vec![state.font.clone().into(), font_size.into()],
                );
            } else {
                // No cached image, fall back to text rendering
                let text = parsed_font.encode_text(&c.to_string());
                writer.add_operation(
                    TextOperations::ShowText,
                    vec![Object::String(tux_pdf_low::types::PdfString::Hexadecimal(
                        text,
                    ))],
                );
                cursor.x += glyph_width;
                total_width += glyph_width;
            }

            max_height = max_height.max(glyph_height);
        }

        Ok(Size {
            width: total_width,
            height: max_height,
        })
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
        cursor: &mut TextCursor,
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
            let item_size = item.write(current_state, writer, cursor)?;
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
