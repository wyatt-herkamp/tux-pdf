use derive_more::derive::From;

use crate::{
    graphics::{
        color::Color, shapes::RectangleStyleType, size::Size, styles::Padding, GraphicStyles,
    },
    units::Pt,
    utils::{IsEmpty, Merge},
};

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub enum TableColumnMinWidth {
    /// The column width is fixed and will not be recalculated
    Fixed(Pt),
    /// The column width is a percentage of the total width
    Percentage(f32),
    /// The column width set to the whatever is left after the other columns have been calculated
    /// Good for a notes column
    ///
    /// You are limited to one column with this setting
    AutoFill,
}
#[derive(Debug, Clone, Copy, PartialEq, From)]
pub enum TableColumnMaxWidth {
    /// The column width is fixed and will not be recalculated
    Fixed(Pt),
    /// The column width is a percentage of the available width
    Percentage(f32),
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridColumnRules {
    pub min_width: Option<TableColumnMinWidth>,
    pub max_width: Option<TableColumnMaxWidth>,
}
/// A column in the table
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct GridBuilderColumn {
    pub width: Pt,
    pub x: Pt,
    pub rules: GridColumnRules,
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridColumn {
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
#[derive(Debug, Clone, PartialEq)]
pub struct TableRow {
    pub content_y: Pt,
    pub border_line_y: Pt,
    pub height: Pt,
    pub styles: Option<GridStyleGroup>,
}
/// A new Grid Column based on the the size of the column header text
#[derive(Debug, Clone, PartialEq)]
pub struct NewTableColumn {
    pub initial_size: Size,
    pub rules: GridColumnRules,
}
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
    /// The styles for the cells
    pub cell_styles: Option<GridStyleGroup>,
    /// The styles for the rows
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
