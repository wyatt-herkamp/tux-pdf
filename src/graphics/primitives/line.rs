use tux_pdf_low::types::Object;

use crate::{
    document::PdfResources,
    graphics::{points_to_object_array, OperationWriter, PdfObjectType},
    units::Pt,
    utils::copy_into,
};

use super::{PathConstructionOperators, PathPaintOperationKeys, PdfPosition};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StraightLine<U = Pt> {
    pub start: PdfPosition<U>,
    /// 2D Points for the line. The `bool` indicates whether the next point is a bezier control point.
    pub points: Vec<PdfPosition<U>>,
    /// Whether the line should automatically be closed
    pub is_closed: bool,
}
impl<T> From<Vec<T>> for StraightLine
where
    T: Into<PdfPosition>,
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
    T: Into<PdfPosition>,
{
    fn from((points, is_closed): (Vec<T>, bool)) -> Self {
        let mut points: Vec<PdfPosition> = points.into_iter().map(Into::into).collect();
        let start = points.remove(0);
        Self {
            start,
            points: points.into_iter().map(Into::into).collect(),
            is_closed,
        }
    }
}
impl PdfObjectType for StraightLine {
    fn write(
        self,
        _: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), crate::TuxPdfError> {
        if self.points.is_empty() {
            return Ok(());
        }
        writer.add_operation(PathConstructionOperators::PathMoveTo, self.start.into());

        for point in self.points.iter() {
            let point = *point;
            writer.add_operation(PathConstructionOperators::PathLineTo, point.into());
        }

        if self.is_closed {
            writer.push_empty_op(PathPaintOperationKeys::StrokeClose);
        } else {
            writer.push_empty_op(PathPaintOperationKeys::Stroke);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LinePoint {
    Point(PdfPosition),
    V1Bezier {
        start: PdfPosition,
        end: PdfPosition,
    },
    V2Bezier {
        start: PdfPosition,
        end: PdfPosition,
    },
    /// The first point is the start, the second point is the control point, and the third point is the end.
    ///
    /// See 8.5.2.2
    ThreePointBezier {
        start: PdfPosition,
        end: PdfPosition,
        new_control: PdfPosition,
    },
}
impl Default for LinePoint {
    fn default() -> Self {
        Self::Point(PdfPosition::default())
    }
}

copy_into!(LinePoint => (PathConstructionOperators, Vec<Object>));
impl From<LinePoint> for (PathConstructionOperators, Vec<Object>) {
    fn from(value: LinePoint) -> Self {
        match value {
            LinePoint::Point(point) => (PathConstructionOperators::PathLineTo, point.into()),
            LinePoint::V1Bezier { start, end } => (
                PathConstructionOperators::BezierCurveTwoV1,
                points_to_object_array(&[start, end]),
            ),
            LinePoint::V2Bezier { start, end } => (
                PathConstructionOperators::BezierCurveTwoV2,
                points_to_object_array(&[start, end]),
            ),
            LinePoint::ThreePointBezier {
                start,
                end,
                new_control,
            } => (
                PathConstructionOperators::BezierCurveFour,
                points_to_object_array(&[start, new_control, end]),
            ),
        }
    }
}
impl From<PdfPosition> for LinePoint {
    fn from(point: PdfPosition) -> Self {
        Self::Point(point)
    }
}
impl From<(PdfPosition, PdfPosition)> for LinePoint {
    fn from((start, end): (PdfPosition, PdfPosition)) -> Self {
        Self::V1Bezier { start, end }
    }
}
impl From<(PdfPosition, PdfPosition, PdfPosition)> for LinePoint {
    fn from((start, end, new_control): (PdfPosition, PdfPosition, PdfPosition)) -> Self {
        Self::ThreePointBezier {
            start,
            end,
            new_control,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Line {
    pub start: PdfPosition,
    /// 2D Points for the line. The `bool` indicates whether the next point is a bezier control point.
    pub points: Vec<LinePoint>,
    /// Whether the line should automatically be closed
    pub is_closed: bool,
}

impl PdfObjectType for Line {
    fn write(
        self,
        _: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), crate::TuxPdfError> {
        if self.points.is_empty() {
            return Ok(());
        }
        writer.add_operation(PathConstructionOperators::PathMoveTo, self.start.into());

        for point in self.points.iter() {
            let (key, operands) = point.into();
            writer.add_operation(key, operands);
        }

        if self.is_closed {
            writer.push_empty_op(PathPaintOperationKeys::StrokeClose);
        } else {
            writer.push_empty_op(PathPaintOperationKeys::Stroke);
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
