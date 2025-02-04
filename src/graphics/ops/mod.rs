mod keys;
use crate::{
    document::{LayerId, PdfResources},
    TuxPdfError,
};
pub use keys::*;
use tux_pdf_low::{
    content::Operation,
    types::{Dictionary, Object, PdfType, Stream},
};

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

    fn calculate_number_of_pdf_objects(&self) -> usize {
        match self {
            PdfObject::TextBlock(block) => block.calculate_number_of_pdf_objects(),
            PdfObject::NewLine => 1,
            PdfObject::Graphics(graphic) => graphic.calculate_number_of_pdf_objects(),
            PdfObject::Styles(styles) => styles.calculate_number_of_pdf_objects(),
            PdfObject::Image(pdf_image_operation) => {
                pdf_image_operation.calculate_number_of_pdf_objects()
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct OperationWriter {
    pub(crate) operations: Vec<Operation>,
}
impl From<OperationWriter> for Vec<Operation> {
    fn from(writer: OperationWriter) -> Self {
        writer.operations
    }
}
impl OperationWriter {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            operations: Vec::with_capacity(capacity),
        }
    }
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
    /// Begin a new layer
    ///
    /// Ensure to call [Self::end_section] after this to close the layer
    pub fn start_layer(&mut self, layer_id: LayerId) {
        self.add_operation(
            OperationKeys::BeginLayer,
            vec![Object::name("OC".as_bytes().to_vec()), layer_id.into()],
        );
    }

    /// Begin a marked content section
    ///
    /// Ensure to call [Self::end_section] after this to close the section
    pub fn begin_marked_content(&mut self, section_name: impl Into<String>) {
        let section_name = section_name.into();
        self.add_operation(
            OperationKeys::BeginMarkedContent,
            vec![Object::name(section_name.into_bytes())],
        );
    }
    /// Used for both ending a [layer](Self::start_layer) and a [marked content section](Self::begin_marked_content)
    pub fn end_section(&mut self) {
        self.push_empty_op(OperationKeys::EndSection);
    }

    pub(crate) fn into_stream(self, dictionary: Dictionary) -> Result<Stream, TuxPdfError> {
        let stream_content = self.operations.write_to_vec()?;
        Ok(Stream::new(dictionary, stream_content))
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

    fn calculate_number_of_pdf_objects(&self) -> usize {
        1
    }
}
/// A type that can contain pdf objects such as pages or layers
pub trait LayerType {
    /// Add an object to the layer
    ///
    /// # Errors
    /// Currently this function does not return any errors however a result is used to allow for future expansion
    fn add_to_layer(&mut self, object: impl Into<PdfObject>) -> Result<(), TuxPdfError>;
}
