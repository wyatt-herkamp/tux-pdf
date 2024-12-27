use std::{
    borrow::Cow,
    fmt::Debug,
    ops::{Add, AddAssign, Sub},
};

use crate::{
    document::{FontRef, FontType},
    graphics::{Point, TextStyle},
    units::Pt,
};

use super::shapes::OutlineRect;

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Size<U = Pt> {
    pub width: U,
    pub height: U,
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
    fn render_size(
        &self,
        document: &crate::document::PdfDocument,
        settings: &Self::Settings,
    ) -> Result<Size, crate::TuxPdfError>;
}
impl<T> RenderSize for &T
where
    T: RenderSize,
{
    type Settings = T::Settings;
    fn render_size(
        &self,
        document: &crate::document::PdfDocument,
        settings: &Self::Settings,
    ) -> Result<Size, crate::TuxPdfError> {
        (*self).render_size(document, settings)
    }
}

impl RenderSize for () {
    type Settings = ();
    fn render_size(
        &self,
        _: &crate::document::PdfDocument,
        _: &Self::Settings,
    ) -> Result<Size, crate::TuxPdfError> {
        Ok(Size::default())
    }
}
impl RenderSize for &str {
    type Settings = TextStyle;

    fn render_size(
        &self,
        document: &crate::document::PdfDocument,
        settings: &TextStyle,
    ) -> Result<Size, crate::TuxPdfError> {
        let font = &settings.font_ref;
        match font {
            FontRef::External(font_id) => {
                let font = document
                    .resources
                    .fonts
                    .get_external_font(font_id)
                    .ok_or_else(|| crate::TuxPdfError::FontNotRegistered(font_id.clone()))?;
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

    fn render_size(
        &self,
        document: &crate::document::PdfDocument,
        settings: &TextStyle,
    ) -> Result<Size, crate::TuxPdfError> {
        self.as_str().render_size(document, settings)
    }
}

impl RenderSize for Cow<'_, str> {
    type Settings = TextStyle;
    fn render_size(
        &self,
        document: &crate::document::PdfDocument,
        settings: &Self::Settings,
    ) -> Result<Size, crate::TuxPdfError> {
        self.as_ref().render_size(document, settings)
    }
}
