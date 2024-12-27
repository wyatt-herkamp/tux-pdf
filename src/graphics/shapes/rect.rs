use std::ops::{Add, Sub};

use crate::{
    graphics::{
        size::Size, OperationKeys, OperationWriter, PaintMode, PathConstructionOperators,
        PathPaintOperationKeys, PdfOperationType, Point, Polygon, StraightLine, WindingOrder,
    },
    units::{Pt, UnitType},
    utils::copy_into,
};
#[derive(Debug, Default, PartialEq, Clone)]
pub struct PaintedRect<U = Pt> {
    /// Lower left corner of the rectangle
    pub position: Point<U>,
    pub size: Size<U>,
    pub winding_order: WindingOrder,
    pub paint_mode: PaintMode,
}

impl<U> From<(Point<U>, Size<U>)> for PaintedRect<U>
where
    U: Default,
{
    fn from((position, size): (Point<U>, Size<U>)) -> Self {
        Self {
            position,
            size,
            ..Default::default()
        }
    }
}

impl PaintedRect {
    pub fn new(x: Pt, y: Pt, width: Pt, height: Pt) -> Self {
        let position = Point { x, y };
        let size = Size { width, height };
        Self {
            position,
            size,
            ..Default::default()
        }
    }
    pub fn with_mode(mut self, mode: PaintMode) -> Self {
        self.paint_mode = mode;
        self
    }
    pub fn with_winding_order(mut self, order: WindingOrder) -> Self {
        self.winding_order = order;
        self
    }
    /// Creates a new rectangle with the center point being the center of the rectangle.
    ///
    /// Then it calculates the rest of the points based on the center point and the width and height.
    pub fn new_rectangle(width: Pt, height: Pt, center_point: Point) -> Self {
        let Point { x, y } = center_point;

        let position = Point {
            x: x - width / 2f32.pt(),
            y: y - height / 2f32.pt(),
        };
        let size = Size { width, height };
        Self {
            position,
            size,
            ..Default::default()
        }
    }
    pub fn new_square(size: Pt, center_point: Point) -> Self {
        let Point { x, y } = center_point;
        let position = Point {
            x: x - size / 2f32.pt(),
            y: y - size / 2f32.pt(),
        };
        let size = Size {
            width: size,
            height: size,
        };
        Self {
            position,
            size,
            ..Default::default()
        }
    }
}

impl PdfOperationType for PaintedRect {
    fn write(
        self,
        _: &crate::document::PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), crate::TuxPdfError> {
        let Size { width, height } = self.size;
        let Point { x, y } = self.position;
        writer.add_operation(
            PathConstructionOperators::PathRectangle,
            vec![x.into(), y.into(), width.into(), height.into()],
        );
        writer.add_operation(self.paint_mode.operation_key(self.winding_order), vec![]);
        writer.add_operation(PathPaintOperationKeys::PathPaintEnd, vec![]);

        Ok(())
    }
}

pub struct RectangleStyle {
    /// Curves the corners of the rectangle
    pub top_left_radius: Option<f32>,
    pub top_right_radius: Option<f32>,
    pub bottom_left_radius: Option<f32>,
    pub bottom_right_radius: Option<f32>,
}

pub trait RectangleStyleType {
    fn has_fill_color(&self) -> bool;
    fn has_outline_color(&self) -> bool;

    fn paint_mode(&self) -> Option<PaintMode> {
        if self.has_fill_color() && self.has_outline_color() {
            Some(PaintMode::FillStroke)
        } else if self.has_fill_color() {
            Some(PaintMode::Fill)
        } else if self.has_outline_color() {
            Some(PaintMode::Stroke)
        } else {
            None
        }
    }
}
/// A rectangle that does not have any fill just an outline
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct OutlineRect<U = Pt> {
    pub position: Point<U>,
    pub size: Size<U>,
}
impl PdfOperationType for OutlineRect {
    fn write(
        self,
        resources: &crate::document::PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), crate::TuxPdfError> {
        let line: StraightLine = self.into();
        line.write(resources, writer)
    }
}
impl OutlineRect {
    pub fn media_box(&self) -> OutlineRect {
        *self
    }
    /// Creates a rectangle where the squares center is the center_point
    pub fn new_square(size: Pt, center_point: Point) -> Self {
        let Point { x, y } = center_point;
        let position = Point {
            x: x - size / 2f32.pt(),
            y: y - size / 2f32.pt(),
        };
        let size = Size {
            width: size,
            height: size,
        };
        Self { position, size }
    }
}
copy_into!(OutlineRect => StraightLine);
impl From<OutlineRect> for StraightLine {
    fn from(rect: OutlineRect) -> Self {
        let (start, points) = rect.gen_points_and_start();
        StraightLine {
            start,
            points,
            is_closed: true,
        }
    }
}
copy_into!(OutlineRect => Polygon);
impl From<OutlineRect> for Polygon {
    fn from(rect: OutlineRect) -> Self {
        let points = gen_points_from_position_and_size(rect.position, rect.size)
            .into_iter()
            .map(|point| (point, false))
            .collect();
        Polygon {
            rings: vec![points],
            mode: PaintMode::Fill,
            winding_order: WindingOrder::NonZero,
        }
    }
}

impl<U> OutlineRect<U> {
    pub fn lower_left(&self) -> Point<U>
    where
        U: Copy,
    {
        self.position
    }

    pub fn upper_right(&self) -> Point<U>
    where
        U: Copy + std::ops::Add<Output = U>,
    {
        let Point { x, y } = self.position;
        let Size { width, height } = self.size;
        Point {
            x: x + width,
            y: y + height,
        }
    }
    pub fn from_wh(width: U, height: U) -> Self
    where
        U: Default,
    {
        let size = Size { width, height };
        Self {
            position: Point::default(),
            size,
        }
    }

    fn gen_points_and_start(&self) -> (Point<U>, Vec<Point<U>>)
    where
        U: Copy + Sub<Output = U> + Add<Output = U> + Default,
    {
        gen_start_and_points_from_position_and_size(self.position, self.size)
    }
}

impl OutlineRect {
    pub fn to_array(&self) -> Vec<lopdf::Object> {
        let Point { x, y } = self.position;
        let Size { width, height } = self.size;
        vec![
            (x.0.round() as i64).into(),
            (y.0.round() as i64).into(),
            (width.0.round() as i64).into(),
            (height.0.round() as i64).into(),
        ]
    }
}

impl<U> From<OutlineRect<U>> for lopdf::Object
where
    U: Into<lopdf::Object>,
{
    fn from(rect: OutlineRect<U>) -> Self {
        lopdf::Object::Array(rect.into())
    }
}

impl<U, T> From<OutlineRect<U>> for Vec<T>
where
    U: Into<T>,
{
    fn from(value: OutlineRect<U>) -> Self {
        let Point { x, y } = value.position;
        let Size { width, height } = value.size;
        vec![x.into(), y.into(), width.into(), height.into()]
    }
}

fn gen_start_and_points_from_position_and_size<U>(
    position: Point<U>,
    size: Size<U>,
) -> (Point<U>, Vec<Point<U>>)
where
    U: Copy + Sub<Output = U> + Add<Output = U> + Default,
{
    let Point { x, y } = position;
    let Size { width, height } = size;
    let top = y;
    let bottom = y - height;
    let left = x;
    let right = x + width;

    let tl = Point { x: left, y: top };
    let tr = Point { x: right, y: top };
    let br = Point {
        x: right,
        y: bottom,
    };
    let bl = Point { x: left, y: bottom };
    (tl, vec![tr, br, bl])
}
fn gen_points_from_position_and_size<U>(position: Point<U>, size: Size<U>) -> Vec<Point<U>>
where
    U: Copy + Sub<Output = U> + Add<Output = U> + Default,
{
    let Point { x, y } = position;
    let Size { width, height } = size;
    let top = y;
    let bottom = y - height;
    let left = x;
    let right = x + width;

    let tl = Point { x: left, y: top };
    let tr = Point { x: right, y: top };
    let br = Point {
        x: right,
        y: bottom,
    };
    let bl = Point { x: left, y: bottom };
    vec![tl, tr, br, bl]
}
