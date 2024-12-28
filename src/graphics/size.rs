use std::{
    borrow::Cow,
    convert::Infallible,
    fmt::Debug,
    ops::{Add, AddAssign, Sub},
};

use crate::{
    document::{FontRef, FontType, PdfDocument, ResourceNotRegistered},
    graphics::{Point, TextStyle},
    units::{Mm, Pt, Px},
};

use super::shapes::OutlineRect;

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Size<U = Pt> {
    pub width: U,
    pub height: U,
}
impl Size<Px> {
    /// Converts the current size into a Size with the unit as [Pt]
    pub fn into_pt_with_dpi(self, dpi: f32) -> Size<Pt> {
        Size {
            width: self.width.into_pt_with_dpi(dpi),
            height: self.height.into_pt_with_dpi(dpi),
        }
    }
    /// Converts the current size into a Size with the unit as [Mm]
    pub fn into_mm_with_dpi(self, dpi: f32) -> Size<Mm> {
        Size {
            width: self.width.into_mm_with_dpi(dpi),
            height: self.height.into_mm_with_dpi(dpi),
        }
    }
}
impl<U> Size<U> {
    pub fn scale_width(&self, scale: f32) -> Size<U>
    where
        U: Copy + Into<f32> + From<f32>,
    {
        let width: f32 = self.width.into();
        Size::new(U::from(width * scale), self.height)
    }
    pub fn scale_height(&self, scale: f32) -> Size<U>
    where
        U: Copy + Into<f32> + From<f32>,
    {
        let height: f32 = self.height.into();
        Size::new(self.width, U::from(height * scale))
    }
    pub fn scale(&self, scale_x: f32, scale_y: f32) -> Size<U>
    where
        U: Copy + Into<f32> + From<f32>,
    {
        let width: f32 = self.width.into();
        let height: f32 = self.height.into();
        Size::new(U::from(width * scale_x), U::from(height * scale_y))
    }
}
impl<U> From<Size<U>> for (U, U) {
    fn from(size: Size<U>) -> (U, U) {
        (size.width, size.height)
    }
}
impl<U> Add<Size<U>> for Size<U>
where
    U: Add<Output = U>,
{
    type Output = Size<U>;
    fn add(self, other: Size<U>) -> Size<U> {
        Size::new(self.width + other.width, self.height + other.height)
    }
}
impl<U> AddAssign<Size<U>> for Size<U>
where
    U: AddAssign,
{
    fn add_assign(&mut self, other: Size<U>) {
        self.width += other.width;
        self.height += other.height;
    }
}
impl<U> Sub<Size<U>> for Size<U>
where
    U: Sub<Output = U>,
{
    type Output = Size<U>;
    fn sub(self, other: Size<U>) -> Size<U> {
        Size::new(self.width - other.width, self.height - other.height)
    }
}
impl<U> Sub<(U, U)> for Size<U>
where
    U: Sub<Output = U> + Copy,
{
    type Output = Size<U>;
    fn sub(self, (width, height): (U, U)) -> Size<U> {
        Size::new(self.width - width, self.height - height)
    }
}

impl<U> Size<U> {
    pub const fn new(width: U, height: U) -> Self {
        Self { width, height }
    }
    pub const fn landscape(&self) -> Self
    where
        U: Copy,
    {
        Self::new(self.height, self.width)
    }
    pub fn top_left_point(&self) -> Point<U>
    where
        U: Default + Copy,
    {
        Point {
            x: U::default(),
            y: self.height,
        }
    }
}
impl<U> From<(U, U)> for Size<U> {
    fn from((width, height): (U, U)) -> Self {
        Self { width, height }
    }
}

impl<U> From<Size<U>> for OutlineRect<U>
where
    U: Default,
{
    fn from(size: Size<U>) -> Self {
        OutlineRect {
            position: Point::default(),
            size,
        }
    }
}

pub trait RenderSize {
    type Settings: Debug;
    type Error: std::error::Error;
    fn render_size(
        &self,
        document: &crate::document::PdfDocument,
        settings: &Self::Settings,
    ) -> Result<Size, Self::Error>;
}
impl<T> RenderSize for &T
where
    T: RenderSize,
{
    type Settings = T::Settings;
    type Error = T::Error;
    fn render_size(
        &self,
        document: &crate::document::PdfDocument,
        settings: &Self::Settings,
    ) -> Result<Size, Self::Error> {
        (*self).render_size(document, settings)
    }
}

impl RenderSize for () {
    type Settings = ();
    type Error = Infallible;
    fn render_size(&self, _: &PdfDocument, _: &Self::Settings) -> Result<Size, Infallible> {
        Ok(Size::default())
    }
}
impl RenderSize for &str {
    type Settings = TextStyle;
    type Error = ResourceNotRegistered;

    fn render_size(
        &self,
        document: &PdfDocument,
        settings: &TextStyle,
    ) -> Result<Size, ResourceNotRegistered> {
        let font = &settings.font_ref;
        match font {
            FontRef::External(font_id) => {
                let font = document
                    .resources
                    .fonts
                    .get_external_font(font_id)
                    .ok_or_else(|| ResourceNotRegistered::from(font.clone()))?;
                Ok(font.calculate_size_of_text(self.as_ref(), settings))
            }
            FontRef::Builtin(builtin_font) => {
                Ok(builtin_font.calculate_size_of_text(self.as_ref(), settings))
            }
        }
    }
}
impl RenderSize for String {
    type Settings = TextStyle;
    type Error = ResourceNotRegistered;

    fn render_size(
        &self,
        document: &PdfDocument,
        settings: &TextStyle,
    ) -> Result<Size, ResourceNotRegistered> {
        self.as_str().render_size(document, settings)
    }
}

impl RenderSize for Cow<'_, str> {
    type Settings = TextStyle;
    type Error = ResourceNotRegistered;

    fn render_size(
        &self,
        document: &PdfDocument,
        settings: &Self::Settings,
    ) -> Result<Size, ResourceNotRegistered> {
        self.as_ref().render_size(document, settings)
    }
}
