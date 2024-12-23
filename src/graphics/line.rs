use lopdf::Object;

use crate::{units::Pt, utils::copy_into};

use super::{
    color::Color, points_to_object_array, GraphicStyles, OperationKeys, PdfOperationType, Point,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StraightLine<U = Pt> {
    pub start: Point<U>,
    /// 2D Points for the line. The `bool` indicates whether the next point is a bezier control point.
    pub points: Vec<Point<U>>,
    /// Whether the line should automatically be closed
    pub is_closed: bool,
}
impl<T> From<Vec<T>> for StraightLine
where
    T: Into<Point>,
{
    fn from(points: Vec<T>) -> Self {
        let mut points: Vec<_> = points.into_iter().map(Into::into).collect();
        let start = points.remove(0);
        Self {
            points,
            start,
            ..Default::default()
        }
    }
}
impl<T> From<(Vec<T>, bool)> for StraightLine
where
    T: Into<Point>,
{
    fn from((points, is_closed): (Vec<T>, bool)) -> Self {
        let mut points: Vec<Point> = points.into_iter().map(Into::into).collect();
        let start = points.remove(0);
        Self {
            start,
            points: points.into_iter().map(Into::into).collect(),
            is_closed,
            ..Default::default()
        }
    }
}
impl PdfOperationType for StraightLine {
    fn write(
        &self,
        _: &crate::document::PdfResources,
        writer: &mut super::OperationWriter,
    ) -> Result<(), crate::TuxPdfError> {
        if self.points.is_empty() {
            return Ok(());
        }
        writer.add_operation(OperationKeys::PathMoveTo, self.start.into());

        for point in self.points.iter() {
            let point = *point;
            writer.add_operation(OperationKeys::PathLineTo, point.into());
        }

        if self.is_closed {
            writer.push_empty_op(OperationKeys::PathPaintStrokeClose);
        } else {
            writer.push_empty_op(OperationKeys::PathPaintStroke);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LinePoint {
    Point(Point),
    V1Bezier {
        start: Point,
        end: Point,
    },
    V2Bezier {
        start: Point,
        end: Point,
    },
    /// The first point is the start, the second point is the control point, and the third point is the end.
    ///
    /// See 8.5.2.2
    ThreePointBezier {
        start: Point,
        end: Point,
        new_control: Point,
    },
}
impl Default for LinePoint {
    fn default() -> Self {
        Self::Point(Point::default())
    }
}

copy_into!(LinePoint => (OperationKeys, Vec<Object>));
impl From<LinePoint> for (OperationKeys, Vec<Object>) {
    fn from(value: LinePoint) -> Self {
        match value {
            LinePoint::Point(point) => (OperationKeys::PathLineTo, point.into()),
            LinePoint::V1Bezier { start, end } => (
                OperationKeys::BezierCurveTwoV1,
                points_to_object_array(&[start, end]),
            ),
            LinePoint::V2Bezier { start, end } => (
                OperationKeys::BezierCurveTwoV2,
                points_to_object_array(&[start, end]),
            ),
            LinePoint::ThreePointBezier {
                start,
                end,
                new_control,
            } => (
                OperationKeys::BezierCurveFour,
                points_to_object_array(&[start, new_control, end]),
            ),
        }
    }
}
impl From<Point> for LinePoint {
    fn from(point: Point) -> Self {
        Self::Point(point)
    }
}
impl From<(Point, Point)> for LinePoint {
    fn from((start, end): (Point, Point)) -> Self {
        Self::V1Bezier { start, end }
    }
}
impl From<(Point, Point, Point)> for LinePoint {
    fn from((start, end, new_control): (Point, Point, Point)) -> Self {
        Self::ThreePointBezier {
            start,
            end,
            new_control,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Line {
    pub start: Point,
    /// 2D Points for the line. The `bool` indicates whether the next point is a bezier control point.
    pub points: Vec<LinePoint>,
    /// Whether the line should automatically be closed
    pub is_closed: bool,
}

impl PdfOperationType for Line {
    fn write(
        &self,
        _: &crate::document::PdfResources,
        writer: &mut super::OperationWriter,
    ) -> Result<(), crate::TuxPdfError> {
        if self.points.is_empty() {
            return Ok(());
        }
        writer.add_operation(OperationKeys::PathMoveTo, self.start.into());

        for point in self.points.iter() {
            let (key, operands) = point.into();
            writer.add_operation(key, operands);
        }

        if self.is_closed {
            writer.push_empty_op(OperationKeys::PathPaintStrokeClose);
        } else {
            writer.push_empty_op(OperationKeys::PathPaintStroke);
        }

        Ok(())
    }
}
impl From<StraightLine> for Line {
    fn from(value: StraightLine) -> Self {
        Self {
            start: value.start,
            points: value.points.into_iter().map(LinePoint::from).collect(),
            is_closed: value.is_closed,
        }
    }
}

pub struct LineStyles {
    pub width: Option<f32>,
    pub outline_color: Option<Color>,
    pub outline_thickness: Option<f32>,
}
