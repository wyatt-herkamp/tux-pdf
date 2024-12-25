mod style;

use std::{mem, ops::Deref};

use crate::{
    document::{FontRef, FontRenderSizeParams, InternalFontType, InternalFontTypes, PdfResources},
    graphics::{OperationKeys, Point},
    units::Pt,
    TuxPdfError,
};
use lopdf::Object;
pub use style::*;
use tracing::debug;

use super::{size::Size, OperationWriter, PdfOperation, PdfOperationType};
/// A text item is a string with a list of modifiers
#[derive(Debug, Clone, PartialEq)]
pub struct TextItem {
    pub text: String,
    pub modifiers: Vec<TextModifier>,
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
    pub fn with_position(mut self, position: Point) -> Self {
        self.modifiers.push(TextModifier::Position(position));
        self
    }
    /// Splits the text into two items once the available width is reached
    ///
    /// Returns the remaining text if the text was split
    fn cut_off_at_max(
        &mut self,
        availalble_width: Pt,
        current_state: &TextBlockState,
        resources: &PdfResources,
    ) -> Result<(Pt, Option<Self>), TuxPdfError> {
        let state = state_from_modifiers(&self.modifiers, current_state, resources)?
            .unwrap_or_else(|| current_state.clone());
        let text_render_params = FontRenderSizeParams {
            font_size: current_state.font_size,
        };
        let text_size = state
            .font_type
            .calculate_size_of_text(&self.text, text_render_params);
        if text_size.width < availalble_width {
            return Ok((text_size.width, None));
        }

        let mut new_text = String::new();

        let mut width = Pt::default();

        for c in self.text.chars() {
            let char_size = current_state
                .font_type
                .size_of_char(c, text_render_params)
                .unwrap_or_default();
            if width + char_size.width > availalble_width {
                break;
            }
            width += char_size.width;
            new_text.push(c);
        }

        let remaining_text = self.text[new_text.len()..].to_string();
        self.text = new_text;
        let new_line = Some(Self {
            text: remaining_text,
            modifiers: self.modifiers.clone(),
        });
        Ok((width, new_line))
    }

    pub(crate) fn calculate_size_of_text(
        &self,
        current_state: &TextBlockState,
        resources: &PdfResources,
    ) -> Result<Size, TuxPdfError> {
        let state: TextBlockState<'_> =
            state_from_modifiers(&self.modifiers, current_state, resources)?
                .unwrap_or_else(|| current_state.clone());
        Ok(state.font_type.calculate_size_of_text(
            &self.text,
            FontRenderSizeParams {
                font_size: current_state.font_size,
            },
        ))
    }
    fn write(
        self,
        current_state: &TextBlockState,
        resources: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<Size, TuxPdfError> {
        let state = write_modifiers(self.modifiers, current_state, resources, writer)?
            .unwrap_or_else(|| current_state.clone());
        let text_size = state.font_type.calculate_size_of_text(
            &self.text,
            FontRenderSizeParams {
                font_size: state.font_size,
            },
        );
        let text = state.font_type.encode_text(self.text);

        writer.add_operation(
            OperationKeys::ShowText,
            vec![Object::String(text, lopdf::StringFormat::Hexadecimal)],
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
#[derive(Debug, Clone, PartialEq)]
pub struct TextLine {
    pub items: Vec<TextItem>,
    pub modifiers: Vec<TextModifier>,
}
impl TextLine {
    fn write(
        self,
        current_state: &TextBlockState,
        resources: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<Size, TuxPdfError> {
        let mut line_size: Size = Size::default();
        write_modifiers(self.modifiers, current_state, resources, writer)?;

        for item in self.items {
            writer.push_empty_op(OperationKeys::SaveGraphicsState);
            let item_size = item.write(current_state, resources, writer)?;
            writer.push_empty_op(OperationKeys::RestoreGraphicsState);

            line_size.width += item_size.width;
            line_size.height = line_size.height.max(item_size.height);
        }

        Ok(line_size)
    }

    fn apply_max_width(
        mut self,
        max_width: Pt,
        current_state: &TextBlockState,
        resources: &PdfResources,
        lines: &mut Vec<TextLine>,
    ) -> Result<LineMaxWidth, TuxPdfError> {
        let mut item_iterator = mem::take(&mut self.items).into_iter();
        // Read Line Items until the max width is reached
        let mut item = item_iterator.next();
        let mut extra_items = Vec::new();

        let mut current_max_width = max_width;
        while let Some(mut current_item) = item {
            let (width, new_item) =
                current_item.cut_off_at_max(current_max_width, current_state, resources)?;
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
            new_line.apply_max_width(max_width, current_state, resources, lines)?;
            Ok(LineMaxWidth::SplitLines)
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextBlockContent(pub Vec<TextLine>);
impl TextBlockContent {
    fn apply_max_width(
        &mut self,
        max_width: Pt,
        current_state: &TextBlockState,
        resources: &PdfResources,
    ) -> Result<(), TuxPdfError> {
        let old_lines = mem::take(&mut self.0);

        for line in old_lines {
            let current_max_width = max_width;
            let result =
                line.apply_max_width(current_max_width, current_state, resources, &mut self.0)?;

            debug!(?result, "Line Max Width Result");
        }
        Ok(())
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
        let mut lines = Vec::new();

        for line in text.lines() {
            let line = TextLine {
                items: vec![TextItem::new(line)],
                modifiers: Vec::new(),
            };
            lines.push(line);
        }
        Self(lines)
    }
}
impl From<&str> for TextBlockContent {
    fn from(text: &str) -> Self {
        let mut lines = Vec::new();

        for line in text.lines() {
            let line = TextLine {
                items: vec![TextItem::new(line)],
                modifiers: Vec::new(),
            };
            lines.push(line);
        }
        Self(lines)
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextBlock {
    /// Each Entry is a line of text
    pub content: TextBlockContent,
    pub style: TextStyle,
    pub position: Point,
}
impl TextBlock {
    fn writer_many(
        lines: Vec<TextLine>,
        current_state: TextBlockState,
        line_spacing: Pt,
        resources: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), TuxPdfError> {
        writer.push_empty_op(OperationKeys::SaveGraphicsState);

        let mut line_iterator = lines.into_iter().peekable();
        while let Some(line) = line_iterator.next() {
            writer.push_empty_op(OperationKeys::SaveGraphicsState);

            let line_size = line.write(&current_state, resources, writer)?;

            writer.push_empty_op(OperationKeys::RestoreGraphicsState);
            if line_iterator.peek().is_some() {
                let line_height = -(line_size.height + line_spacing);

                debug!(?line_height, "Line Height");
                writer.add_operation(
                    OperationKeys::TextPosition,
                    Point {
                        x: Pt::default(),
                        y: line_height,
                    }
                    .into(),
                );
            }
        }

        writer.push_empty_op(OperationKeys::RestoreGraphicsState);
        Ok(())
    }
    fn write_one(
        current_state: TextBlockState,
        content: TextLine,
        resources: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), TuxPdfError> {
        content.write(&current_state, resources, writer)?;
        Ok(())
    }
}
impl PdfOperationType for TextBlock {
    fn write(
        self,
        resources: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), TuxPdfError> {
        let Self {
            content: mut lines,
            style,
            position,
        } = self;
        if lines.is_empty() {
            return Ok(());
        }

        writer.push_empty_op(OperationKeys::BeginText);
        writer.add_operation(OperationKeys::TextPosition, position.into());

        let font = resources
            .fonts
            .internal_font_type(&style.font_ref)
            .ok_or(TuxPdfError::from(style.font_ref.clone()))?;
        let writer_state = TextBlockState {
            font: style.font_ref.clone(),
            font_size: style.font_size,
            font_type: font,
        };
        if let Some(max_width) = style.max_width {
            lines.apply_max_width(max_width, &writer_state, resources)?;
            debug!(?lines, "Lines after applying max width");
        }
        let line_spacing = style.line_spacing.unwrap_or_default();
        style.write(resources, writer)?;

        if lines.len() > 1 {
            Self::writer_many(lines.0, writer_state, line_spacing, resources, writer)?;
        } else {
            let line = lines.0.remove(0);
            Self::write_one(writer_state, line, resources, writer)?;
        }

        writer.push_empty_op(OperationKeys::EndText);
        Ok(())
    }
}
impl From<TextBlock> for PdfOperation {
    fn from(text: TextBlock) -> Self {
        PdfOperation::TextBlock(text)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TextBlockState<'resources> {
    font: FontRef,
    font_size: Pt,
    font_type: InternalFontTypes<'resources>,
}

#[cfg(test)]
mod tests {
    use crate::{
        document::{owned_ttf_parser::OwnedPdfTtfFont, PdfDocument},
        graphics::Point,
        page::{page_sizes::A4, PdfPage},
        tests::{init_logger, test_fonts_directory},
        units::UnitType,
    };

    use super::{TextBlock, TextStyle};
    #[test]
    fn max_width_test() -> anyhow::Result<()> {
        init_logger();
        let mut doc = PdfDocument::new("Table Test");
        let roboto_font_reader = std::fs::File::open(
            test_fonts_directory()
                .join("Roboto")
                .join("Roboto-Regular.ttf"),
        )?;
        let roboto_font = OwnedPdfTtfFont::new_from_reader(roboto_font_reader, 0)?;

        let roboto = doc.font_map().register_external_font(roboto_font)?;
        let text_block = TextBlock {
            content: "This is a test of the emergency broadcast system. This is only a test."
                .into(),
            style: TextStyle {
                font_ref: roboto,
                font_size: 12f32.pt(),
                max_width: Some(100f32.pt()),
                ..Default::default()
            },
            position: Point {
                x: 10f32.into(),
                y: 210f32.into(),
            },
        };
        let mut page = PdfPage::new_from_page_size(A4);
        page.add_operation(text_block.into());
        doc.add_page(page);
        let mut pdf = doc.write_to_lopdf_document()?;

        let mut file = std::fs::File::create("target/test_max_width.pdf")?;

        pdf.save_to(&mut file)?;
        Ok(())
    }
}
