mod group;
pub use group::*;
pub mod high;
mod ops;
mod style;
use std::{
    borrow::Cow,
    fmt::Debug,
    marker::PhantomData,
    ops::{Add, Sub},
};
pub use style::*;
pub mod styles;
use color::{Color, ColorWriter};
use lopdf::Object;
pub use ops::*;
pub mod color;
pub mod layouts;
pub mod size;
pub mod table;
use crate::{document::FontRef, units::Pt};
mod line;
pub mod text;
pub use line::*;
pub use text::*;
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Rotation(pub i64);
impl From<Rotation> for Object {
    fn from(rotation: Rotation) -> Object {
        rotation.0.into()
    }
}

#[derive(Debug, Copy, Default, Clone, PartialEq)]
pub struct Point<P = Pt> {
    /// x position from the bottom left corner in pt
    pub x: P,
    /// y position from the bottom left corner in pt
    pub y: P,
}
impl<P> Add<Point<P>> for Point<P>
where
    P: Add<Output = P>,
{
    type Output = Point<P>;
    fn add(self, other: Point<P>) -> Point<P> {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl<P> Sub<Point<P>> for Point<P>
where
    P: Sub<Output = P>,
{
    type Output = Point<P>;
    fn sub(self, other: Point<P>) -> Point<P> {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl<P> From<Point<P>> for Vec<Object>
where
    P: Into<Object>,
{
    #[inline]
    fn from(point: Point<P>) -> Vec<Object> {
        vec![point.x.into(), point.y.into()]
    }
}
impl<P> From<&Point<P>> for Vec<Object>
where
    P: Into<Object> + Copy,
{
    fn from(point: &Point<P>) -> Vec<Object> {
        vec![point.x.into(), point.y.into()]
    }
}
impl<P> From<Point<P>> for [Object; 2]
where
    P: Into<Object>,
{
    fn from(point: Point<P>) -> [Object; 2] {
        [point.x.into(), point.y.into()]
    }
}
impl<P> From<&Point<P>> for [Object; 2]
where
    P: Into<Object> + Copy,
{
    fn from(point: &Point<P>) -> [Object; 2] {
        [point.x.into(), point.y.into()]
    }
}
/// Converts a slice of points into an array of objects
///
/// So [Point { x: 1, y: 2 }, Point { x: 3, y: 4 }] would become [1, 2, 3, 4]
pub fn points_to_object_array<P>(points: &[Point<P>]) -> Vec<Object>
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Polygon {
    /// 2D Points for the line. The `bool` indicates whether the next point is a bezier control point.
    pub rings: Vec<Vec<(Point, bool)>>,
    /// What type of polygon is this?
    pub mode: PaintMode,
    /// Winding order to use for constructing this polygon
    pub winding_order: WindingOrder,
}
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PaintMode {
    Clip,
    #[default]
    Fill,
    Stroke,
    FillStroke,
}
impl PaintMode {
    pub fn operation_key(&self, winding_order: WindingOrder) -> OperationKeys {
        match (self, winding_order) {
            (PaintMode::Clip, WindingOrder::EvenOdd) => OperationKeys::PathPaintClipEvenOdd,
            (PaintMode::Clip, WindingOrder::NonZero) => OperationKeys::PathPaintClipNonZero,
            (PaintMode::Fill, WindingOrder::EvenOdd) => OperationKeys::PathPaintFillEvenOdd,
            (PaintMode::Fill, WindingOrder::NonZero) => OperationKeys::PathPaintFillNonZero,
            (PaintMode::Stroke, WindingOrder::EvenOdd) => {
                OperationKeys::PathPaintFillStrokeCloseEvenOdd
            }
            (PaintMode::Stroke, WindingOrder::NonZero) => {
                OperationKeys::PathPaintFillStrokeCloseNonZero
            }
            (PaintMode::FillStroke, WindingOrder::EvenOdd) => {
                OperationKeys::PathPaintFillStrokeEvenOdd
            }
            (PaintMode::FillStroke, WindingOrder::NonZero) => {
                OperationKeys::PathPaintFillStrokeNonZero
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindingOrder {
    EvenOdd,
    #[default]
    NonZero,
}
