use size::{ColumnMaxWidth, ColumnMinWidth};

use crate::{
    graphics::{GraphicStyles, Padding, color::Color, shapes::RectangleStyleType},
    units::Pt,
    utils::{IsEmpty, Merge},
};
pub mod size;
/// A group of styles that can be applied to part of the grid
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridStyleGroup {
    pub border_color: Option<Color>,
    pub border_width: Option<Pt>,
    pub background_color: Option<Color>,
}
impl IsEmpty for GridStyleGroup {
    fn is_empty(&self) -> bool {
        self.border_color.is_none()
            && self.border_width.is_none()
            && self.background_color.is_none()
    }
}
impl Merge for GridStyleGroup {
    fn merge(&mut self, other: Self) {
        if let Some(border_color) = other.border_color {
            self.border_color = Some(border_color);
        }
        if let Some(border_width) = other.border_width {
            self.border_width = Some(border_width);
        }
        if let Some(background_color) = other.background_color {
            self.background_color = Some(background_color);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridStyles {
    pub cell_content_padding: Padding,

    /// The outermost object styles.So the lines around the table
    pub outer_styles: Option<GridStyleGroup>,
    /// The styles applied to all cells.
    pub cell_styles: Option<GridStyleGroup>,
    /// The styles applied to all rows
    pub row_styles: Option<GridStyleGroup>,
}
impl From<GridStyleGroup> for GraphicStyles {
    fn from(value: GridStyleGroup) -> Self {
        Self {
            line_width: value.border_width,
            fill_color: value.background_color,
            outline_color: value.border_color,
        }
    }
}
impl From<&GridStyleGroup> for GraphicStyles {
    fn from(value: &GridStyleGroup) -> Self {
        Self {
            line_width: value.border_width,
            fill_color: value.background_color.clone(),
            outline_color: value.border_color.clone(),
        }
    }
}

impl RectangleStyleType for GridStyleGroup {
    fn has_fill_color(&self) -> bool {
        self.background_color.is_some()
    }

    fn has_outline_color(&self) -> bool {
        self.border_color.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridColumnRules {
    pub min_width: Option<ColumnMinWidth>,
    pub max_width: Option<ColumnMaxWidth>,
}
/// A column in the table
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GridBuilderColumn {
    pub width: Pt,
    pub x: Pt,
    pub rules: GridColumnRules,
}
impl Default for GridBuilderColumn {
    fn default() -> Self {
        Self {
            width: Pt::default(),
            x: Pt::default(),
            rules: GridColumnRules::default(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridCell {
    pub width: Pt,
    pub width_no_padding: Pt,
    pub x: Pt,
    pub content_x: Pt,
    pub border_line_x: Pt,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridCellStylingRules {
    pub fill_color: Option<Color>,
    pub outline_color: Option<Color>,
    pub outline_width: Option<Pt>,
}
/// A row in the table
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct GridBuilderRow {
    pub y: Pt,
    pub content_start_y: Pt,
    pub height: Pt,
    pub styles: Option<GridStyleGroup>,
}
