mod group;
pub use group::*;
mod ops;
pub mod shapes;
use std::{
    fmt::Debug,
    ops::{Add, Sub},
};
pub mod styles;
use lopdf::Object;
pub use ops::*;
pub use styles::*;
pub mod color;
pub mod layouts;
pub mod size;
pub mod table;
use crate::units::{Mm, Pt};
pub mod image;
mod line;
pub mod text;
pub use line::*;
pub use text::*;
pub mod ctm;
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Rotation(pub i64);
impl From<Rotation> for Object {
    fn from(rotation: Rotation) -> Object {
        rotation.0.into()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Point<P = Pt> {
    /// x position from the bottom left corner in pt
    pub x: P,
    /// y position from the bottom left corner in pt
    pub y: P,
}
impl<P> Point<P> {
    pub fn new(x: P, y: P) -> Self {
        Self { x, y }
    }
}
impl Point<Pt> {
    pub fn into_mm(self) -> Point<Mm> {
        Point {
            x: self.x.into(),
            y: self.y.into(),
        }
    }
}
impl Point<Mm> {
    pub fn into_pt(self) -> Point<Pt> {
        Point {
            x: self.x.into(),
            y: self.y.into(),
        }
    }
}
impl<U> Default for Point<U>
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
    /// Clip the current path
    Clip,
    /// Fill the current path
    #[default]
    Fill,
    /// Stroke the current path
    Stroke,
    /// Fill and then stroke the current path
    FillStroke,
}
impl PaintMode {
    pub fn operation_key(&self, winding_order: WindingOrder) -> PathPaintOperationKeys {
        match (self, winding_order) {
            (PaintMode::Clip, WindingOrder::EvenOdd) => PathPaintOperationKeys::ClipEvenOdd,
            (PaintMode::Clip, WindingOrder::NonZero) => PathPaintOperationKeys::ClipNonZero,
            (PaintMode::Fill, WindingOrder::EvenOdd) => PathPaintOperationKeys::FillEvenOdd,
            (PaintMode::Fill, WindingOrder::NonZero) => PathPaintOperationKeys::FillNonZero,
            (PaintMode::Stroke, _) => PathPaintOperationKeys::Stroke,
            (PaintMode::FillStroke, WindingOrder::EvenOdd) => {
                PathPaintOperationKeys::FillStrokeEvenOdd
            }
            (PaintMode::FillStroke, WindingOrder::NonZero) => {
                PathPaintOperationKeys::FillStrokeNonZero
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

operation_keys!(PathPaintOperationKeys => {
    /// Stroke Close
    StrokeClose => "s",
    /// Path Paint Stroke
    Stroke => "S",
    /// Fills the path using the non-zero winding rule
    ///
    /// Any subpaths that are open are implicitly closed
    ///
    /// See [WindingOrder::NonZero]
    FillNonZero => "f",
    /// Fills the path using the even-odd rule
    ///
    /// See  [WindingOrder::EvenOdd]
    FillEvenOdd => "f*",
    /// Fill and then strokes the path. Using the non zero winding number rule
    ///
    /// This has the same result as constructing two identical path objects. One using F then the second one using S
    FillStrokeNonZero => "B",
    /// Fill and then strokes the path. Using the even odd winding number rule
    ///
    /// This operator shall produce the same result as [PathPaintOperationKeys::FillStrokeNonZero] except the path is filled with [PathPaintOperationKeys::FillEvenOdd] instead of [PathPaintOperationKeys::FillNonZero]
    FillStrokeEvenOdd => "B*",
    /// Fill, stroke, and then close the path. Using the non zero winding number rule
    FillStrokeCloseNonZero => "b",
    /// Fill, stroke, and then close the path. Using the even odd winding number rule
    FillStrokeCloseEvenOdd => "b*",
    /// Modify the current clipping path by interescting it with the current path using the non zero winding number rule
    ClipNonZero => "W",
    /// Modify the current clipping path by interescting it with the current path using the even odd winding number rule
    ClipEvenOdd => "W*",
    /// End the path oject without filling or stroking it
    PathPaintEnd => "n"
});

operation_keys!(PathConstructionOperators => {
    /// Path move to
    PathMoveTo => "m",
    /// Path Line To
    PathLineTo => "l",
    /// Cubic Bezier with two points in V1
    ///
    /// Look at Figure 17 in the spec
    ///
    /// But `v` makes it curve closer to the second point
    BezierCurveTwoV1 => "v",
    /// Cubic Bezier with two points in V2
    ///
    /// Look at Figure 17 in the spec
    ///
    /// But `y` makes it curve closer to the first point
    BezierCurveTwoV2 => "y",
    /// Cubic Bezier with 3 points
    ///
    /// Look at Figure 16 in the spec
    ///
    /// But `c` makes it curve in the middle and sets the new current point to the last point
    BezierCurveFour => "c",
    /// Append a rectnagle to the current path as a complete subpath
    ///
    /// ## Parameters
    /// x - The x coordinate of the lower-left corner of the rectangle
    /// y - The y coordinate of the lower-left corner of the rectangle
    /// width - The width of the rectangle
    /// height - The height of the rectangle
    PathRectangle => "re"
});
