use derive_more::derive::From;

use crate::{document::PdfResources, TuxPdfError};

use super::{
    primitives::{Line, StraightLine},
    shapes::{OutlineRect, PaintedRect},
    GraphicStyles, OperationKeys, OperationWriter, PdfObject, PdfObjectType,
};

/// By default every graphic item you add to be rendered will end with a call to restore the graphics state.
///
/// By putting them all in a graphics group, you can avoid this by calling `restore` only once at the end of the group.
/// allowing them to share the same graphics state.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GraphicsGroup {
    pub styles: Option<GraphicStyles>,
    pub items: Vec<GraphicItems>,
}
impl<Item, Iter> From<Iter> for GraphicsGroup
where
    Iter: Iterator<Item = Item>,
    Item: Into<GraphicItems>,
{
    fn from(group: Iter) -> Self {
        Self {
            styles: None,
            items: group.map(|item| item.into()).collect(),
        }
    }
}

impl GraphicsGroup {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_from_iter<Item, Iter>(&mut self, group: Iter)
    where
        Iter: Iterator<Item = Item>,
        Item: Into<GraphicItems>,
    {
        self.items.reserve(group.size_hint().0);
        for item in group {
            self.add_item(item.into());
        }
    }
    pub fn with_styles(styles: GraphicStyles) -> Self {
        Self {
            styles: Some(styles),
            items: Vec::new(),
        }
    }
    pub fn add_item<I>(&mut self, item: I)
    where
        I: Into<GraphicItems>,
    {
        self.items.push(item.into());
    }
}
impl PdfObjectType for GraphicsGroup {
    fn write(
        self,
        resources: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), TuxPdfError> {
        writer.add_operation(OperationKeys::SaveGraphicsState, vec![]);
        if let Some(styles) = self.styles {
            styles.write(resources, writer)?;
        }
        for item in self.items {
            writer.add_operation(OperationKeys::SaveGraphicsState, vec![]);
            item.write(resources, writer)?;
            writer.add_operation(OperationKeys::RestoreGraphicsState, vec![]);
        }
        writer.add_operation(OperationKeys::RestoreGraphicsState, vec![]);

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, From)]
pub enum GraphicItems {
    StraightLine(StraightLine),
    Line(Line),
    Rectangle(PaintedRect),
    OutlineRectangle(OutlineRect),
    Group(GraphicsGroup),
}

impl<I> From<I> for PdfObject
where
    I: Into<GraphicItems>,
{
    fn from(item: I) -> Self {
        PdfObject::Graphics(item.into())
    }
}
impl PdfObjectType for GraphicItems {
    fn write(
        self,
        resources: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), TuxPdfError> {
        match self {
            GraphicItems::StraightLine(line) => line.write(resources, writer),
            GraphicItems::Line(line) => line.write(resources, writer),
            GraphicItems::Rectangle(rect) => rect.write(resources, writer),
            GraphicItems::OutlineRectangle(rect) => rect.write(resources, writer),
            GraphicItems::Group(group) => group.write(resources, writer),
        }
    }
}
