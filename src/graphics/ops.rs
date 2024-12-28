use lopdf::{
    content::{Content, Operation},
    Object,
};
mod keys;
use crate::{document::PdfResources, TuxPdfError};
pub use keys::*;

use super::{
    group::GraphicItems, image::PdfImageOperation, GraphicStyles, TextBlock, TextOperations,
};
/// Operations that can occur in a PDF page
#[derive(Debug, Clone, PartialEq)]
pub enum PdfOperation {
    TextBlock(TextBlock),
    NewLine,
    Graphics(GraphicItems),
    Styles(GraphicStyles),
    Image(PdfImageOperation),
}

impl PdfOperationType for PdfOperation {
    fn write(
        self,
        resources: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), TuxPdfError> {
        match self {
            PdfOperation::TextBlock(block) => {
                block.write(resources, writer)?;
            }
            PdfOperation::NewLine => {
                writer.add_operation(TextOperations::TextNewLine, vec![]);
            }
            PdfOperation::Graphics(graphic) => {
                graphic.write(resources, writer)?;
            }
            PdfOperation::Styles(styles) => {
                styles.write(resources, writer)?;
            }
            PdfOperation::Image(pdf_image_operation) => {
                pdf_image_operation.write(resources, writer)?;
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
    pub fn add_operation(&mut self, operation: impl OperationKeyType, operands: Vec<Object>) {
        self.operations.push(operation.to_operation(operands));
    }
    pub fn push_empty_op(&mut self, operation: impl OperationKeyType) {
        self.operations.push(operation.no_operand());
    }
    pub fn operations(self) -> Vec<Operation> {
        self.operations
    }
    #[inline(always)]
    pub fn save_graphics_state(&mut self) {
        self.push_empty_op(OperationKeys::SaveGraphicsState);
    }
    #[inline(always)]
    pub fn restore_graphics_state(&mut self) {
        self.push_empty_op(OperationKeys::RestoreGraphicsState);
    }
}

pub trait PdfOperationType {
    fn write(
        self,
        resources: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), TuxPdfError>;
}
