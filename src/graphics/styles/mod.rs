mod margin;
mod padding;
pub use margin::*;
pub use padding::*;
use std::borrow::Cow;
use std::ops::Add;

use crate::{
    units::Pt,
    utils::{IsEmpty, PartailOrFull, PartialStruct},
};

use super::{
    color::{Color, ColorWriter},
    shapes::RectangleStyleType,
    OperationKeys, OperationWriter, PdfOperation, PdfOperationType,
};
pub(crate) fn add_two_optional<U>(a: Option<U>, b: Option<U>) -> Option<U>
where
    U: Add<Output = U> + Copy,
{
    match (a, b) {
        (Some(a), Some(b)) => Some(a + b),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GraphicStyles {
    pub line_width: Option<Pt>,
    pub fill_color: Option<Color>,
    pub outline_color: Option<Color>,
}
impl RectangleStyleType for GraphicStyles {
    fn has_fill_color(&self) -> bool {
        self.fill_color.is_some()
    }
    fn has_outline_color(&self) -> bool {
        self.outline_color.is_some()
    }
}
impl From<GraphicStyles> for PdfOperation {
    fn from(value: GraphicStyles) -> Self {
        PdfOperation::Styles(value)
    }
}

impl PdfOperationType for GraphicStyles {
    fn write(
        self,
        resources: &crate::document::PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), crate::TuxPdfError> {
        if let Some(width) = self.line_width {
            writer.add_operation(OperationKeys::SetLineWidth, vec![width.into()]);
        }
        let color_writer = ColorWriter {
            outline_color: self.outline_color.map(Cow::Owned),
            fill_color: self.fill_color.map(Cow::Owned),
        };
        color_writer.write(resources, writer)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct PartialGraphicStyles {
    pub line_width: Option<Pt>,
    pub fill_color: Option<Color>,
    pub outline_color: Option<Color>,
}
impl IsEmpty for PartialGraphicStyles {
    fn is_empty(&self) -> bool {
        self.line_width.is_none() && self.fill_color.is_none() && self.outline_color.is_none()
    }
}
impl PdfOperationType for PartialGraphicStyles {
    fn write(
        self,
        resources: &crate::document::PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), crate::TuxPdfError> {
        if let Some(width) = self.line_width {
            writer.add_operation(OperationKeys::SetLineWidth, vec![width.into()]);
        }
        let color_writer = ColorWriter {
            outline_color: self.outline_color.map(Cow::Owned),
            fill_color: self.fill_color.map(Cow::Owned),
        };
        color_writer.write(resources, writer)?;
        Ok(())
    }
}
pub type PartialOrFullGraphicStyle = PartailOrFull<PartialGraphicStyles>;
impl PartialStruct for PartialGraphicStyles {
    type FullStruct = GraphicStyles;

    fn merge_with_full<'full>(
        &self,
        full: &'full Self::FullStruct,
    ) -> Cow<'full, Self::FullStruct> {
        if self.is_empty() {
            return Cow::Borrowed(full);
        }
        let mut new = full.clone();
        if let Some(line_width) = self.line_width {
            new.line_width = Some(line_width);
        }
        if let Some(fill_color) = self.fill_color.clone() {
            new.fill_color = Some(fill_color);
        }
        if let Some(outline_color) = self.outline_color.clone() {
            new.outline_color = Some(outline_color);
        }
        Cow::Owned(new)
    }
}
