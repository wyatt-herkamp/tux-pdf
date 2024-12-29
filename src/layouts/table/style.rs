use crate::{
    graphics::{
        color::{Color, BLACK_RGB, GRAY_RGB},
        styles::Padding,
        PartialOrFullTextStyle, TextStyle,
    },
    layouts::table::{GridStyleGroup, TableColumnMaxWidth},
    units::{Pt, UnitType},
    utils::Merge,
};

use super::builder::TableColumnMinWidth;
/// A cell is where the area where the row and column intersect
///
/// Currently cell styles are not supported
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CellStyle {
    pub fill_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: Option<Pt>,
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ColumnStyle {
    /// Will set the maximum width of the column
    pub max_width: Option<TableColumnMaxWidth>,
    /// Will set the minimum width of the column
    pub min_width: Option<TableColumnMinWidth>,
    /// Cell Styling Options
    pub cell_styles: Option<CellStyle>,
}
/// Row Styles are the styles that are applied to the entire row
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RowStyles {
    pub text_style: Option<PartialOrFullTextStyle>,
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: Option<Pt>,
}
impl From<RowStyles> for GridStyleGroup {
    fn from(row_styles: RowStyles) -> Self {
        Self {
            border_color: row_styles.border_color,
            border_width: row_styles.border_width,
            background_color: row_styles.background_color,
        }
    }
}
impl From<&RowStyles> for GridStyleGroup {
    fn from(row_styles: &RowStyles) -> Self {
        Self {
            border_color: row_styles.border_color.clone(),
            border_width: row_styles.border_width,
            background_color: row_styles.background_color.clone(),
        }
    }
}
impl Merge for RowStyles {
    fn merge(&mut self, other: Self) {
        if let Some(other) = other.text_style {
            self.text_style = Some(other);
        }
        if let Some(other) = other.background_color {
            self.background_color = Some(other);
        }
        if let Some(other) = other.border_color {
            self.border_color = Some(other);
        }
        if let Some(other) = other.border_width {
            self.border_width = Some(other);
        }
    }
}
impl Merge<RowStyles> for GridStyleGroup {
    fn merge(&mut self, other: RowStyles) {
        if let Some(other) = other.background_color {
            self.background_color = Some(other);
        }
        if let Some(other) = other.border_color {
            self.border_color = Some(other);
        }
        if let Some(other) = other.border_width {
            self.border_width = Some(other);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableStyles {
    /// Overrides for styles for the header
    pub header_styles: Option<RowStyles>,
    /// Base Text Styles
    ///
    /// Any overrides will inherit from this
    pub text_styles: TextStyle,
    /// If less than what is required it will be ignored.
    /// This is used if you want to have extra space
    pub min_row_height: Option<Pt>,
    /// The padding around the cell content
    ///
    /// Recommended to be at least 5pt in all directions
    pub cell_content_padding: Padding,
    /// The outermost object styles.
    /// So the lines around the table
    pub outer_styles: Option<GridStyleGroup>,
    /// The styles for the cells
    ///
    /// If None then no extra styles will be applied to each cell
    pub cell_styles: Option<GridStyleGroup>,
    /// Base Row Styles. Any overrides will inherit from this
    pub row_styles: GridStyleGroup,
    /// If true then the header will be repeated on a new page
    pub repeat_header_on_new_page: bool,
}
impl Default for TableStyles {
    fn default() -> Self {
        Self {
            header_styles: None,
            text_styles: Default::default(),
            cell_content_padding: Padding::all(5f32.pt()),
            outer_styles: Some(GridStyleGroup {
                border_color: Some(BLACK_RGB),
                border_width: Some(1f32.pt()),
                ..Default::default()
            }),
            row_styles: GridStyleGroup {
                background_color: Some(GRAY_RGB),
                border_color: Some(BLACK_RGB),
                border_width: Some(1f32.pt()),
            },
            repeat_header_on_new_page: true,
            cell_styles: None,
            min_row_height: None,
        }
    }
}
