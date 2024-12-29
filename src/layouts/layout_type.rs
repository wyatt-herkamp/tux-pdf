use crate::{
    document::PdfDocument,
    graphics::{
        image::PdfImage, size::Size, BlankSpace, HasPosition, LayerType, PdfPosition, TextBlock,
    },
    units::Pt,
    TuxPdfError,
};

use super::LayoutError;
/// A layout item is a item that lives within a layout
pub trait LayoutItemType: HasPosition {
    /// Minimum size of the layout item if it has one
    fn min_size(&self) -> Option<Size<Option<Pt>>> {
        None
    }
    /// Set the minimum size of the layout item
    fn set_min_size(&mut self, _size: Size<Option<Pt>>) -> Result<(), TuxPdfError> {
        Err(LayoutError::UnableToResize.into())
    }

    /// Maximum size of the layout item if it has one
    fn max_size(&self) -> Option<Size<Option<Pt>>> {
        None
    }
    /// Set the maximum size of the layout item
    fn set_max_size(&mut self, _size: Size<Option<Pt>>) -> Result<(), TuxPdfError> {
        Err(LayoutError::UnableToResize.into())
    }
    fn can_resize(&self) -> bool {
        false
    }

    fn resize(&mut self, _new_size: Size<Option<Pt>>) -> Result<(), TuxPdfError> {
        Err(LayoutError::UnableToResize.into())
    }
    fn calculate_size(&mut self, document: &PdfDocument) -> Result<Size, TuxPdfError>;
    /// Can the layout item be resized?
    fn render<L: LayerType>(self, document: &PdfDocument, page: &mut L) -> Result<(), TuxPdfError>
    where
        Self: Sized;
}
/// A layout item is a single item that can be placed on a page
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutItem {
    #[cfg(feature = "taffy")]
    TaffyLayout(Box<super::taffy_layout::PdfTaffyLayout>),
    Text(TextBlock),
    Image(PdfImage),
    BlankSpace(BlankSpace),
}
macro_rules! from {
    (
        $(
            $type:ty => $variant:ident
         ),*
    ) => {
        $(
            impl From<$type> for LayoutItem {
                fn from(item: $type) -> Self {
                    LayoutItem::$variant(item)
                }
            }
        )*
    };
    (
        $(
            $type:ty => boxed($variant:ident),
         )*
    ) => {
        $(
            impl From<$type> for LayoutItem {
                fn from(item: $type) -> Self {
                    LayoutItem::$variant(Box::new(item))
                }
            }
            from! {
                Box<$type> => $variant
            }
        )*
    };
}
from! {
    TextBlock => Text,
    PdfImage => Image,
    BlankSpace => BlankSpace
}
#[cfg(feature = "taffy")]
from! {
    super::taffy_layout::PdfTaffyLayout => boxed(TaffyLayout),
}

impl HasPosition for LayoutItem {
    fn position(&self) -> PdfPosition {
        match self {
            LayoutItem::Text(text) => text.position(),
            LayoutItem::Image(image) => image.position(),
            LayoutItem::BlankSpace(blank_space) => blank_space.position(),
            #[cfg(feature = "taffy")]
            LayoutItem::TaffyLayout(layout) => layout.position(),
        }
    }

    fn set_position(&mut self, position: PdfPosition) {
        match self {
            LayoutItem::Text(text) => text.set_position(position),
            LayoutItem::Image(image) => image.set_position(position),
            LayoutItem::BlankSpace(blank_space) => blank_space.set_position(position),
            #[cfg(feature = "taffy")]
            LayoutItem::TaffyLayout(layout) => layout.set_position(position),
        }
    }
}
impl LayoutItemType for LayoutItem {
    fn calculate_size(&mut self, document: &PdfDocument) -> Result<Size, TuxPdfError> {
        match self {
            LayoutItem::Text(text) => text.calculate_size(document),
            LayoutItem::Image(image) => image.calculate_size(document),
            LayoutItem::BlankSpace(bs) => bs.calculate_size(document),
            #[cfg(feature = "taffy")]
            LayoutItem::TaffyLayout(layout) => layout.calculate_size(document),
        }
    }

    fn render<L: LayerType>(self, document: &PdfDocument, page: &mut L) -> Result<(), TuxPdfError>
    where
        Self: Sized,
    {
        match self {
            LayoutItem::Text(text) => text.render(document, page),
            LayoutItem::Image(image) => image.render(document, page),
            LayoutItem::BlankSpace(bs) => bs.render(document, page),
            #[cfg(feature = "taffy")]
            LayoutItem::TaffyLayout(layout) => layout.render(document, page),
        }
    }
}
