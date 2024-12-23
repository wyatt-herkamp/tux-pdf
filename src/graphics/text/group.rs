use std::borrow::Cow;

use lopdf::Object;

use crate::{
    document::PdfResources,
    graphics::{OperationKeys, OperationWriter, PdfOperation, PdfOperationType, Point},
    TuxPdfError,
};

use super::{encode_text, PartialOrFullTextStyle, TextStyle};

#[derive(Debug, PartialEq, Clone)]
pub enum MultiTextValue<'text> {
    Text {
        value: Cow<'text, str>,
        position: Point,
        styles: Option<PartialOrFullTextStyle>,
    },
    NewLine,
}
#[derive(Debug, PartialEq, Clone)]
pub struct MultiText<'text> {
    pub values: Vec<MultiTextValue<'text>>,
    pub base_style: TextStyle,
}
impl PdfOperationType for MultiText<'_> {
    fn write(
        &self,
        resources: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), TuxPdfError> {
        writer.push_empty_op(OperationKeys::BeginText);
        writer.push_empty_op(OperationKeys::SaveGraphicsState);

        self.base_style.write(resources, writer)?;
        for text in &self.values {
            match text {
                MultiTextValue::NewLine => {
                    writer.push_empty_op(OperationKeys::TextNewLine);
                    continue;
                }
                MultiTextValue::Text {
                    styles,
                    position,
                    value,
                } => {
                    writer.push_empty_op(OperationKeys::SaveGraphicsState);
                    writer.add_operation(OperationKeys::TextPosition, position.into());
                    if let Some(styles) = styles {
                        styles
                            .merge_with_full(&self.base_style)
                            .write(resources, writer)?;
                    }

                    let text = encode_text(resources, value, &self.base_style.font_ref)?;

                    writer.add_operation(
                        OperationKeys::ShowText,
                        vec![Object::String(text, lopdf::StringFormat::Hexadecimal)],
                    );
                    writer.push_empty_op(OperationKeys::RestoreGraphicsState);
                }
            }
        }
        writer.push_empty_op(OperationKeys::RestoreGraphicsState);
        writer.push_empty_op(OperationKeys::EndText);
        Ok(())
    }
}
impl From<MultiText<'static>> for PdfOperation {
    fn from(text: MultiText<'static>) -> Self {
        PdfOperation::MultiText(text)
    }
}
