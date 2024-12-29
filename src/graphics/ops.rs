use lopdf::{
    content::{Content, Operation},
    Object,
};
mod keys;
use crate::{document::PdfResources, TuxPdfError};
pub use keys::*;

use super::{group::GraphicItems, image::PdfImage, GraphicStyles, TextBlock, TextOperations};
/// Operations that can occur in a PDF page
#[derive(Debug, Clone, PartialEq)]
pub enum PdfObject {
    TextBlock(TextBlock),
    NewLine,
    Graphics(GraphicItems),
    Styles(GraphicStyles),
    Image(PdfImage),
}

impl PdfObjectType for PdfObject {
    fn write(
        self,
        resources: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), TuxPdfError> {
        match self {
            PdfObject::TextBlock(block) => {
                block.write(resources, writer)?;
            }
            PdfObject::NewLine => {
                writer.add_operation(TextOperations::TextNewLine, vec![]);
            }
            PdfObject::Graphics(graphic) => {
                graphic.write(resources, writer)?;
            }
            PdfObject::Styles(styles) => {
                styles.write(resources, writer)?;
            }
            PdfObject::Image(pdf_image_operation) => {
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
/// A type that can be written to a pdf containing a few different types of objects
///
/// - Objects to be displayed such as Text, Shapes, Images
/// - Styles to be applied to the objects
/// - Specific operations to be performed on the objects
pub trait PdfObjectType {
    fn write(
        self,
        resources: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), TuxPdfError>;
}
/// A type that can contain pdf objects such as pages or layers
pub trait LayerType {
    /// Add an object to the layer
    ///
    /// # Errors
    /// Currently this function does not return any errors however a result is used to allow for future expansion
    fn add_to_layer(&mut self, object: PdfObject) -> Result<(), TuxPdfError>;
}
