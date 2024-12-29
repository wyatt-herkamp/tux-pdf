use super::{operation_keys, PdfPosition};
pub mod ctm;
mod line;
pub use line::*;
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Polygon {
    /// 2D Points for the line. The `bool` indicates whether the next point is a bezier control point.
    pub rings: Vec<Vec<(PdfPosition, bool)>>,
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
