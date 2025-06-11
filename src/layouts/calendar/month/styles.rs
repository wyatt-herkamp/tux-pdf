use crate::{
    graphics::{Padding, PartialOrFullTextStyle, TextStyle},
    layouts::{table_grid::style::GridStyleGroup, table::RowStyles},
};
#[derive(Debug, Clone, PartialEq, Default)]
pub enum DatePosition {
    TopLeft,
    #[default]
    TopRight,
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MonthCalendarStyle {
    pub header_styles: Option<RowStyles>,
    pub date_position: DatePosition,
    pub date_styles: Option<PartialOrFullTextStyle>,
    pub text_styles: TextStyle,
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
}
