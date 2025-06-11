use crate::units::Pt;

use super::style::GridStyleGroup;

#[derive(Debug, Clone, PartialEq)]
pub struct SizedGridRow {
    pub content_y: Pt,
    pub border_line_y: Pt,
    pub height: Pt,
    pub styles: Option<GridStyleGroup>,
}
