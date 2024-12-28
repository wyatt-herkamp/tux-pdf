use thiserror::Error;

use crate::{document::PdfDocument, page::PdfPage, TuxPdfError};

use super::{image::PdfImageOperation, size::Size, HasPosition, Point, TextBlock};

pub mod table;
pub mod taffy_layout;
#[derive(Debug, Clone, PartialEq, Error)]
pub enum LayoutError {
    #[error("Unable to resize the item")]
    UnableToResize,
    #[error(transparent)]
    TaffyError(#[from] taffy::TaffyError),
}
pub trait LayoutItemType: HasPosition {
    fn calculate_size(&self, document: &PdfDocument) -> Result<Size, TuxPdfError>;

    fn can_resize(&self) -> bool {
        false
    }

    fn resize(&mut self, _new_size: Size) -> Result<(), TuxPdfError> {
        Err(LayoutError::UnableToResize.into())
    }

    fn render(self, document: &PdfDocument, page: &mut PdfPage) -> Result<(), TuxPdfError>;
}
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutItem {
    Group(LayoutGroup),
    Text(TextBlock),
    Image(PdfImageOperation),
    BlankSpace,
}
impl From<TextBlock> for LayoutItem {
    fn from(text: TextBlock) -> Self {
        LayoutItem::Text(text)
    }
}
impl From<PdfImageOperation> for LayoutItem {
    fn from(image: PdfImageOperation) -> Self {
        LayoutItem::Image(image)
    }
}
impl HasPosition for LayoutItem {
    fn position(&self) -> Point {
        match self {
            LayoutItem::Group(group) => group.position(),
            LayoutItem::Text(text) => text.position(),
            LayoutItem::Image(image) => image.position(),
            LayoutItem::BlankSpace => Point::default(),
        }
    }

    fn set_position(&mut self, position: Point) {
        match self {
            LayoutItem::Group(group) => group.set_position(position),
            LayoutItem::Text(text) => text.set_position(position),
            LayoutItem::Image(image) => image.set_position(position),
            LayoutItem::BlankSpace => {}
        }
    }
}
impl LayoutItemType for LayoutItem {
    fn calculate_size(&self, document: &PdfDocument) -> Result<Size, TuxPdfError> {
        match self {
            LayoutItem::Group(_) => todo!(),
            LayoutItem::Text(text) => text.calculate_size(document),
            LayoutItem::Image(image) => image.calculate_size(document),
            LayoutItem::BlankSpace => Ok(Size::default()),
        }
    }

    fn render(self, document: &PdfDocument, page: &mut PdfPage) -> Result<(), TuxPdfError> {
        match self {
            LayoutItem::Group(_) => todo!(),
            LayoutItem::Text(text) => text.render(document, page),
            LayoutItem::Image(image) => image.render(document, page),
            LayoutItem::BlankSpace => Ok(()),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutGroup {
    items: Vec<LayoutItem>,
    position: Point,
}
impl HasPosition for LayoutGroup {
    fn position(&self) -> Point {
        self.position
    }

    fn set_position(&mut self, position: Point) {
        self.position = position;
    }
}
