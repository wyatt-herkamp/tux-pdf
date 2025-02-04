use crate::{
    document::{ResourceNotRegistered, XObjectId, XObjectRef},
    layouts::LayoutItemType,
    units::{Pt, Px},
    TuxPdfError,
};

use super::{
    primitives::ctm::CurTransMat, size::Size, HasPosition, LayerType, PdfObject, PdfObjectType,
    PdfPosition,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ImageTransform<U = Pt> {
    pub position: PdfPosition<U>,
    /// Rotate (counter-clockwise) around a point, in degree angles
    pub rotate: Option<ImageRotation>,
    pub scale_x: Option<f32>,
    pub scale_y: Option<f32>,
    /// DPI of the image, default is 300.0
    pub dpi: f32,
}

impl<U> ImageTransform<U> {
    pub fn dpi(&self) -> f32 {
        self.dpi
    }
    pub fn scales_or_default(&self) -> (f32, f32) {
        let scale_x = self.scale_x.unwrap_or(1.0);
        let scale_y = self.scale_y.unwrap_or(1.0);
        (scale_x, scale_y)
    }
    pub fn has_scale(&self) -> bool {
        self.scale_x.is_some() || self.scale_y.is_some()
    }
}
impl<U> Default for ImageTransform<U>
where
    U: Default,
{
    fn default() -> Self {
        Self {
            position: PdfPosition::default(),
            rotate: None,
            scale_x: None,
            scale_y: None,
            dpi: 300.0,
        }
    }
}
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct ImageRotation<U = Px> {
    pub angle_ccw_degrees: f32,
    pub rotation_center_x: U,
    pub rotation_center_y: U,
}

/// This is a struct used to show an image on the page/layer of the PDF
#[derive(Debug, Clone, PartialEq)]
pub struct PdfImage {
    pub image: XObjectId,
    pub transform: ImageTransform<Pt>,
}
impl LayoutItemType for PdfImage {
    fn calculate_size(
        &mut self,
        document: &crate::document::PdfDocument,
    ) -> Result<Size, TuxPdfError> {
        let Some(x_object_ref) = document.resources.xobjects.get_xobject(&self.image) else {
            return Err(ResourceNotRegistered::from(self.image.clone()).into());
        };
        let XObjectRef::Image(image) = x_object_ref else {
            return Err(TuxPdfError::InvalidReference("Image"));
        };

        let scaled_size = self.scaled_size(image.image.size);
        Ok(scaled_size)
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
impl HasPosition for PdfImage {
    fn position(&self) -> PdfPosition {
        self.transform.position
    }

    fn set_position(&mut self, position: PdfPosition) {
        self.transform.position = position;
    }
}
impl From<PdfImage> for PdfObject {
    fn from(image_op: PdfImage) -> Self {
        Self::Image(image_op)
    }
}
impl From<XObjectId> for PdfImage {
    fn from(image_ref: XObjectId) -> Self {
        Self {
            image: image_ref,
            transform: ImageTransform::default(),
        }
    }
}
impl PdfObjectType for PdfImage {
    fn write(
        self,
        resources: &crate::document::PdfResources,
        writer: &mut crate::graphics::OperationWriter,
    ) -> Result<(), TuxPdfError> {
        let Some(x_object_ref) = resources.xobjects.get_xobject(&self.image) else {
            return Err(ResourceNotRegistered::from(self.image).into());
        };
        let XObjectRef::Image(image) = x_object_ref else {
            return Err(TuxPdfError::InvalidReference("Image"));
        };

        self.transforms(image.image.size).write(resources, writer)?;

        writer.add_operation(
            crate::graphics::OperationKeys::PaintXObject,
            vec![self.image.into()],
        );
        writer.restore_graphics_state();
        Ok(())
    }
}

impl PdfImage {
    pub fn new(image_ref: XObjectId) -> Self {
        Self {
            image: image_ref,
            transform: ImageTransform::default(),
        }
    }
    pub fn dpi(&self) -> f32 {
        self.transform.dpi
    }
    pub fn with_position(mut self, position: PdfPosition<Pt>) -> Self {
        self.transform.position = position;
        self
    }
    pub fn with_scape(mut self, scale_x: f32, scale_y: f32) -> Self {
        self.transform.scale_x = Some(scale_x);
        self.transform.scale_y = Some(scale_y);
        self
    }
    pub fn with_transform(mut self, transform: ImageTransform<Pt>) -> Self {
        self.transform = transform;
        self
    }
    /// Set the DPI of the image
    pub fn set_dpi(&mut self, dpi: f32) {
        self.transform.dpi = dpi;
    }
    pub fn with_dpi(mut self, dpi: f32) -> Self {
        self.set_dpi(dpi);
        self
    }
    pub fn set_position(&mut self, position: PdfPosition<Pt>) {
        self.transform.position = position;
    }

    /// Returns the scaled size of the image
    ///
    /// After the DPI is applied, the image is scaled by the `scale_x` and `scale_y` values.
    pub fn scaled_size(&self, size: Size<Px>) -> Size<Pt> {
        let size: Size = size.into_pt_with_dpi(self.dpi());

        match (self.transform.scale_x, self.transform.scale_y) {
            (Some(scale_x), Some(scale_y)) => size.scale(scale_x, scale_y),
            (Some(scale_x), None) => size.scale_width(scale_x),
            (None, Some(scale_y)) => size.scale_height(scale_y),
            (None, None) => size,
        }
    }
    /// Get the transforms for the image
    pub fn transforms(&self, size: Size<Px>) -> Vec<CurTransMat> {
        let scaled_size = self.scaled_size(size);
        let transforms = vec![
            CurTransMat::Scale(scaled_size.width, scaled_size.height),
            CurTransMat::Position(self.transform.position),
        ];

        transforms
    }
}
