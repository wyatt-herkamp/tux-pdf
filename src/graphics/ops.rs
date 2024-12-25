use lopdf::{
    content::{Content, Operation},
    Object,
};
mod keys;
use crate::{document::PdfResources, TuxPdfError};
pub use keys::*;

use super::{group::GraphicItems, GraphicStyles, TextBlock};
/// Operations that can occur in a PDF page
#[derive(Debug, Clone, PartialEq)]
pub enum PdfOperation {
    /// Debugging or section marker (arbitrary id can mark a certain point in a stream of operations)
    Marker {
        id: String,
    },
    TextBlock(TextBlock),
    LineBreak,
    Graphics(GraphicItems),
    Styles(GraphicStyles),
}

impl PdfOperationType for PdfOperation {
    fn write(
        self,
        resources: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), TuxPdfError> {
        match self {
            PdfOperation::Marker { id } => {
                writer.add_operation(OperationKeys::BeginText, vec![]);
                writer.add_operation(
                    OperationKeys::TextPosition,
                    vec![Object::Integer(0), Object::Integer(0)],
                );
                writer.add_operation(
                    OperationKeys::ShowText,
                    vec![Object::String(
                        id.as_bytes().to_vec(),
                        lopdf::StringFormat::Hexadecimal,
                    )],
                );
                writer.add_operation(OperationKeys::EndText, vec![]);
            }

            PdfOperation::TextBlock(block) => {
                block.write(resources, writer)?;
            }
            PdfOperation::LineBreak => {
                writer.add_operation(
                    OperationKeys::TextPosition,
                    vec![Object::Integer(0), Object::Integer(-1)],
                );
            }
            PdfOperation::Graphics(graphic) => {
                graphic.write(resources, writer)?;
            }
            PdfOperation::Styles(styles) => {
                styles.write(resources, writer)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct OperationWriter {
    operations: Vec<Operation>,
}
impl From<OperationWriter> for Content {
    fn from(writer: OperationWriter) -> Self {
        Content {
            operations: writer.operations,
        }
    }
}
impl OperationWriter {
    pub fn add_operation(&mut self, operation: OperationKeys, operands: Vec<Object>) {
        self.operations.push(operation.lo_pdf_operation(operands));
    }
    pub fn push_empty_op(&mut self, operation: OperationKeys) {
        self.operations.push(operation.lo_pdf_operation(vec![]));
    }
    pub fn operations(self) -> Vec<Operation> {
        self.operations
    }
}

pub trait PdfOperationType {
    fn write(
        self,
        resources: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), TuxPdfError>;
}
