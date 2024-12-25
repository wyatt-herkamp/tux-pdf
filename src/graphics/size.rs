use std::{
    borrow::Cow,
    fmt::Debug,
    ops::{Add, AddAssign, Sub},
};

use crate::{
    document::{FontRef, FontRenderSizeParams, InternalFontType},
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

pub trait SimpleRenderSize {
    type Settings: Debug;
    fn render_size(
        &self,
        document: &crate::document::PdfDocument,
        settings: &Self::Settings,
    ) -> Result<Size, crate::TuxPdfError>;
}
impl<T> SimpleRenderSize for &T
where
    T: SimpleRenderSize,
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

impl SimpleRenderSize for () {
    type Settings = ();
    fn render_size(
        &self,
        _: &crate::document::PdfDocument,
        _: &Self::Settings,
    ) -> Result<Size, crate::TuxPdfError> {
        Ok(Size::default())
    }
}
impl SimpleRenderSize for &str {
    type Settings = TextStyle;

    fn render_size(
        &self,
        document: &crate::document::PdfDocument,
        settings: &TextStyle,
    ) -> Result<Size, crate::TuxPdfError> {
        let font = &settings.font_ref;
        let font_size = settings.font_size;
        let font_options = FontRenderSizeParams { font_size };
        match font {
            FontRef::External(font_id) => {
                let font = document
                    .resources
                    .fonts
                    .get_external_font(font_id)
                    .ok_or_else(|| crate::TuxPdfError::FontNotRegistered(font_id.clone()))?;
                Ok(font.calculate_size_of_text(self.as_ref(), font_options))
            }
            FontRef::Builtin(builtin_font) => {
                Ok(builtin_font.calculate_size_of_text(self.as_ref(), font_options))
            }
        }
    }
}
impl SimpleRenderSize for String {
    type Settings = TextStyle;

    fn render_size(
        &self,
        document: &crate::document::PdfDocument,
        settings: &TextStyle,
    ) -> Result<Size, crate::TuxPdfError> {
        self.as_str().render_size(document, settings)
    }
}

impl SimpleRenderSize for Cow<'_, str> {
    type Settings = TextStyle;
    fn render_size(
        &self,
        document: &crate::document::PdfDocument,
        settings: &Self::Settings,
    ) -> Result<Size, crate::TuxPdfError> {
        self.as_ref().render_size(document, settings)
    }
}

pub trait RenderSizeObject {
    fn render_size(
        &self,
        document: &crate::document::PdfDocument,
    ) -> Result<Size, crate::TuxPdfError>;

    fn allows_max_width(&self) -> bool;

    fn set_max_width(&mut self, max_width: Pt) -> Result<(), crate::TuxPdfError>;
}
