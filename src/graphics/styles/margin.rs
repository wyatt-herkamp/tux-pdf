use std::ops::Add;

use crate::units::Pt;
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Margin<U = Pt> {
    pub left: Option<U>,
    pub right: Option<U>,
    pub top: Option<U>,
    pub bottom: Option<U>,
}
impl<U> From<Margin<U>> for (Option<U>, Option<U>, Option<U>, Option<U>) {
    fn from(margin: Margin<U>) -> (Option<U>, Option<U>, Option<U>, Option<U>) {
        (margin.left, margin.right, margin.top, margin.bottom)
    }
}
impl<U> From<Margin<U>> for (U, U, U, U)
where
    U: Default,
{
    fn from(margin: Margin<U>) -> (U, U, U, U) {
        (
            margin.left.unwrap_or_default(),
            margin.right.unwrap_or_default(),
            margin.top.unwrap_or_default(),
            margin.bottom.unwrap_or_default(),
        )
    }
}
impl<U> Margin<U> {
    pub fn new(left: U, right: U, top: U, bottom: U) -> Self {
        Self {
            left: Some(left),
            right: Some(right),
            top: Some(top),
            bottom: Some(bottom),
        }
    }
    pub fn all(padding: U) -> Self
    where
        U: Copy,
    {
        Self {
            left: Some(padding),
            right: Some(padding),
            top: Some(padding),
            bottom: Some(padding),
        }
    }
    pub fn horizontal(horizontal: U) -> Self
    where
        U: Copy + Default,
    {
        Self {
            left: Some(horizontal),
            right: Some(horizontal),
            ..Default::default()
        }
    }
    pub fn left_and_right(left: U, right: U) -> Self
    where
        U: Default,
    {
        Self {
            left: Some(left),
            right: Some(right),
            ..Default::default()
        }
    }
    pub fn vertical(vertical: U) -> Self
    where
        U: Copy + Default,
    {
        Self {
            top: Some(vertical),
            bottom: Some(vertical),
            ..Default::default()
        }
    }
    pub fn horizontal_value(&self) -> Option<U>
    where
        U: Copy + Add<Output = U>,
    {
        super::add_two_optional(self.left, self.right)
    }
    pub fn horizontal_or_default(&self, default: U) -> U
    where
        U: Copy + Add<Output = U> + Default,
    {
        self.horizontal_value().unwrap_or(default)
    }
    pub fn vertical_value(&self) -> Option<U>
    where
        U: Copy + Add<Output = U>,
    {
        super::add_two_optional(self.top, self.bottom)
    }
    pub fn vertical_or_default(&self, default: U) -> U
    where
        U: Copy + Add<Output = U> + Default,
    {
        self.vertical_value().unwrap_or(default)
    }
}
