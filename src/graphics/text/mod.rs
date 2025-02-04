mod content;
pub(crate) mod state;
mod style;
pub use content::*;
pub use style::*;

use crate::{
    document::PdfResources,
    graphics::{OperationKeys, PdfPosition},
    units::Pt,
    TuxPdfError,
};
use state::TextBlockState;
use tracing::debug;

use super::{
    super::layouts::LayoutItemType,
    operation_keys,
    size::{RenderSize, Size},
    HasPosition, LayerType, OperationWriter, PdfObject, PdfObjectType,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextBlock {
    /// Each Entry is a line of text
    pub content: TextBlockContent,
    pub style: TextStyle,

    /// The position of the text block
    ///
    /// ## Note
    ///
    /// If the text block contains multiple lines,
    /// This is just the starting position of the text block
    pub position: PdfPosition,
}
impl LayoutItemType for TextBlock {
    fn calculate_size(
        &mut self,
        document: &crate::document::PdfDocument,
    ) -> Result<Size, TuxPdfError> {
        self.content.render_size(document, &self.style)
    }
    fn can_resize(&self) -> bool {
        true
    }
    fn max_size(&self) -> Option<Size<Option<Pt>>> {
        Some(Size {
            width: Some(Pt::default()),
            height: None,
        })
    }

    fn set_max_size(&mut self, size: Size<Option<Pt>>) -> Result<(), TuxPdfError> {
        self.style.max_width = size.width;
        Ok(())
    }

    fn render<L: LayerType>(
        self,
        _: &crate::document::PdfDocument,
        page: &mut L,
    ) -> Result<(), TuxPdfError>
    where
        Self: Sized,
    {
        page.add_to_layer(self)
    }
}
impl HasPosition for TextBlock {
    fn position(&self) -> PdfPosition {
        self.position
    }

    fn set_position(&mut self, position: PdfPosition) {
        self.position = position;
    }
}
impl<T> From<T> for TextBlock
where
    T: Into<TextBlockContent>,
{
    fn from(content: T) -> Self {
        Self {
            content: content.into(),
            ..Default::default()
        }
    }
}

impl TextBlock {
    pub fn with_style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }
    pub fn with_position(mut self, position: PdfPosition) -> Self {
        self.position = position;
        self
    }
    pub fn with_font(mut self, font: crate::document::FontRef) -> Self {
        self.style.font_ref = font;
        self
    }
    pub fn with_font_size(mut self, font_size: Pt) -> Self {
        self.style.font_size = font_size;
        self
    }
    fn writer_many(
        lines: Vec<TextLine>,
        current_state: TextBlockState,
        line_spacing: Pt,
        writer: &mut OperationWriter,
    ) -> Result<(), TuxPdfError> {
        writer.push_empty_op(OperationKeys::SaveGraphicsState);
        // We reverse the array because the starting position is at the bottom left so we need to start from the bottom
        let mut line_iterator = lines.into_iter().rev().peekable();
        while let Some(line) = line_iterator.next() {
            let restore = if !line.modifiers.is_empty() {
                writer.push_empty_op(OperationKeys::SaveGraphicsState);
                true
            } else {
                false
            };
            let line_size = line.write(&current_state, writer)?;
            if restore {
                writer.push_empty_op(OperationKeys::RestoreGraphicsState);
            }
            if line_iterator.peek().is_some() {
                let line_height = line_size.height + line_spacing;

                debug!(?line_height, "Line Height");
                writer.add_operation(
                    TextOperations::TextPosition,
                    PdfPosition {
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
        writer: &mut OperationWriter,
    ) -> Result<(), TuxPdfError> {
        content.write(&current_state, writer)?;
        Ok(())
    }
}
impl PdfObjectType for TextBlock {
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

        writer.push_empty_op(TextOperations::BeginText);
        writer.add_operation(TextOperations::TextPosition, position.into());

        let writer_state = TextBlockState::new(resources, &style)?;
        if let Some(max_width) = style.max_width {
            lines.apply_max_width_inner(max_width, &writer_state)?;
            debug!(?lines, "Lines after applying max width");
        }
        let line_spacing = style.line_spacing.unwrap_or_default();
        style.write(resources, writer)?;

        if lines.len() > 1 {
            Self::writer_many(lines.0, writer_state, line_spacing, writer)?;
        } else {
            let line = lines.0.remove(0);
            Self::write_one(writer_state, line, writer)?;
        }

        writer.push_empty_op(TextOperations::EndText);
        Ok(())
    }
}
impl From<TextBlock> for PdfObject {
    fn from(text: TextBlock) -> Self {
        PdfObject::TextBlock(text)
    }
}

operation_keys!(TextOperations => {
        /// Begin Text
        BeginText => "BT",
        /// Text Font
        TextFont => "Tf",
        /// Text Position
        TextPosition => "Td",
        /// Character Space
        CharacterSpace => "Tc",
        /// Word Space
        WordSpace => "Tw",
        /// Text Rise
        TextRise => "Ts",
        /// Text New Line
        TextNewLine => "T*",
        /// End Text
        EndText => "ET",
        /// Show Text
        ShowText => "Tj"
});
#[cfg(test)]
mod tests {
    use crate::{
        document::{owned_ttf_parser::OwnedPdfTtfFont, PdfDocument},
        graphics::{LayerType, PdfPosition},
        page::{page_sizes::A4, PdfPage},
        tests::{fonts_dir, init_logger},
        units::UnitType,
    };

    use super::{TextBlock, TextStyle};
    #[test]
    fn max_width_test() -> anyhow::Result<()> {
        init_logger();
        let mut doc = PdfDocument::new("Table Test");
        let roboto_font_reader =
            std::fs::File::open(fonts_dir().join("Roboto").join("Roboto-Regular.ttf"))?;
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
            position: A4.top_left_point()
                + PdfPosition {
                    x: 10f32.pt(),
                    y: -15f32.pt(),
                },
        };
        let mut page = PdfPage::new_from_page_size(A4);
        page.add_to_layer(text_block)?;
        doc.add_page(page);
        let pdf = doc.write_into_pdf_document_writer()?;

        let mut file = std::fs::File::create("target/test_max_width.pdf")?;

        pdf.save(&mut file)?;
        Ok(())
    }
}
