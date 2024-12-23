use std::ops::Add;

use crate::units::Pt;
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Padding<U = Pt> {
    pub left: Option<U>,
    pub right: Option<U>,
    pub top: Option<U>,
    pub bottom: Option<U>,
}
impl<U> Padding<U> {
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
    pub fn vertical_value(&self) -> Option<U>
    where
        U: Copy + Add<Output = U>,
    {
        super::add_two_optional(self.top, self.bottom)
    }
}
