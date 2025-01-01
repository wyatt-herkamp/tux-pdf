use std::ops::{Add, Sub};

use tux_pdf_low::types::Object;

use crate::units::{Mm, Pt};

use super::size::Size;

pub trait HasPosition {
    fn position(&self) -> PdfPosition;

    fn set_position(&mut self, position: PdfPosition);

    fn with_position(mut self, position: PdfPosition) -> Self
    where
        Self: Sized,
    {
        self.set_position(position);
        self
    }
}
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct PdfPosition<U = Pt> {
    /// x position from the bottom left corner in pt
    pub x: U,
    /// y position from the bottom left corner in pt
    pub y: U,
}
impl<P> PdfPosition<P> {
    /// Because PDF 0,0 is at the bottom left, this function inverts the y-axis
    pub fn invert_from_page_size(&self, page_size: Size<P>) -> PdfPosition<P>
    where
        P: Copy + Sub<P, Output = P>,
    {
        PdfPosition::<P> {
            x: self.x,
            y: page_size.height - self.y,
        }
    }
}
impl Sub<PdfPosition> for PdfPosition {
    type Output = PdfPosition;
    fn sub(self, other: PdfPosition) -> PdfPosition {
        PdfPosition {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
#[cfg(feature = "taffy")]
mod taffy_point {
    use crate::units::Pt;

    use super::PdfPosition;
    use taffy::Point as TaffyPoint;

    impl From<PdfPosition> for TaffyPoint<f32> {
        fn from(point: PdfPosition) -> TaffyPoint<f32> {
            TaffyPoint {
                x: point.x.0,
                y: point.y.0,
            }
        }
    }
    impl From<TaffyPoint<f32>> for PdfPosition {
        fn from(point: TaffyPoint<f32>) -> PdfPosition {
            PdfPosition {
                x: Pt(point.x),
                y: Pt(point.y),
            }
        }
    }
}
impl<P> PdfPosition<P> {
    pub fn new(x: P, y: P) -> Self {
        Self { x, y }
    }
}
impl PdfPosition<Pt> {
    pub fn into_mm(self) -> PdfPosition<Mm> {
        PdfPosition {
            x: self.x.into(),
            y: self.y.into(),
        }
    }
}
impl PdfPosition<Mm> {
    pub fn into_pt(self) -> PdfPosition<Pt> {
        PdfPosition {
            x: self.x.into(),
            y: self.y.into(),
        }
    }
}
impl<U> Default for PdfPosition<U>
where
    U: Default,
{
    fn default() -> Self {
        Self {
            x: U::default(),
            y: U::default(),
        }
    }
}
impl<P> Add<PdfPosition<P>> for PdfPosition<P>
where
    P: Add<Output = P>,
{
    type Output = PdfPosition<P>;
    fn add(self, other: PdfPosition<P>) -> PdfPosition<P> {
        PdfPosition {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<P> From<PdfPosition<P>> for Vec<Object>
where
    P: Into<Object>,
{
    #[inline]
    fn from(point: PdfPosition<P>) -> Vec<Object> {
        vec![point.x.into(), point.y.into()]
    }
}
impl<P> From<&PdfPosition<P>> for Vec<Object>
where
    P: Into<Object> + Copy,
{
    fn from(point: &PdfPosition<P>) -> Vec<Object> {
        vec![point.x.into(), point.y.into()]
    }
}
impl<P> From<PdfPosition<P>> for [Object; 2]
where
    P: Into<Object>,
{
    fn from(point: PdfPosition<P>) -> [Object; 2] {
        [point.x.into(), point.y.into()]
    }
}
impl<P> From<&PdfPosition<P>> for [Object; 2]
where
    P: Into<Object> + Copy,
{
    fn from(point: &PdfPosition<P>) -> [Object; 2] {
        [point.x.into(), point.y.into()]
    }
}
/// Converts a slice of points into an array of objects
///
/// So [Point { x: 1, y: 2 }, Point { x: 3, y: 4 }] would become [1, 2, 3, 4]
pub fn points_to_object_array<P>(points: &[PdfPosition<P>]) -> Vec<Object>
where
    P: Into<Object> + Copy,
{
    let mut arr = Vec::with_capacity(points.len() * 2);
    for point in points {
        arr.push(point.x.into());
        arr.push(point.y.into());
    }
    arr
}
