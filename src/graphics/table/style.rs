use crate::{
    graphics::{
        color::{Color, BLACK_RGB, GRAY_RGB},
        layouts::grid::{GridStyleGroup, GridStyles},
        styles::Padding,
        LineStyles, PartialOrFullTextStyle, TextStyle,
    },
    units::{Pt, UnitType},
};
/// A cell is where the area where the row and column intersect
///
/// Currently cell styles are not supported
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CellStyle {
    pub text_style: Option<PartialOrFullTextStyle>,
    pub fill_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: Option<Pt>,
}
/// Row Styles are the styles that are applied to the entire row
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RowStyles {
    pub text_style: Option<PartialOrFullTextStyle>,
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: Option<Pt>,
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
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AlternatingRowStyles {
    pub even_row_styles: RowStyles,
    pub odd_row_styles: RowStyles,
}
impl AlternatingRowStyles {
    pub fn row_for_index(&self, index: usize) -> &RowStyles {
        if index % 2 == 0 {
            &self.even_row_styles
        } else {
            &self.odd_row_styles
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum TableRowColoring {
    Alternating {
        alternating_styles: Box<AlternatingRowStyles>,
    },
    Single {
        row_style: Box<RowStyles>,
    },
}
impl Default for TableRowColoring {
    fn default() -> Self {
        Self::Single {
            row_style: Box::new(RowStyles {
                border_width: Some(1f32.pt()),
                border_color: Some(BLACK_RGB),
                background_color: Some(GRAY_RGB),
                ..Default::default()
            }),
        }
    }
}
impl From<RowStyles> for TableRowColoring {
    fn from(row_style: RowStyles) -> Self {
        Self::Single {
            row_style: Box::new(row_style),
        }
    }
}
impl From<AlternatingRowStyles> for TableRowColoring {
    fn from(alternating_styles: AlternatingRowStyles) -> Self {
        Self::Alternating {
            alternating_styles: Box::new(alternating_styles),
        }
    }
}
impl From<(RowStyles, RowStyles)> for TableRowColoring {
    fn from((even_row_styles, odd_row_styles): (RowStyles, RowStyles)) -> Self {
        Self::Alternating {
            alternating_styles: Box::new(AlternatingRowStyles {
                even_row_styles,
                odd_row_styles,
            }),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnStyles {
    pub column_line_color: Option<Color>,
    pub column_line_width: Option<Pt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableStyles {
    /// Overrides for styles for the header
    pub header_styles: Option<RowStyles>,
    pub text_styles: TextStyle,
    pub grid_styles: GridStyles,
    /// If less than what is required it will be ignored.
    ///  This is used if you want to have extra space
    pub min_row_height: Option<Pt>,

    pub row_colors: TableRowColoring,

    pub column_styles: Option<ColumnStyles>,

    pub repeat_header_on_new_page: bool,
}
impl Default for TableStyles {
    fn default() -> Self {
        Self {
            header_styles: None,
            text_styles: Default::default(),
            grid_styles: GridStyles {
                cell_content_padding: Padding::all(5f32.pt()),
                outer_styles: Some(GridStyleGroup {
                    border_color: Some(BLACK_RGB),
                    border_width: Some(1f32.pt()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            min_row_height: None,
            row_colors: Default::default(),
            column_styles: None,
            repeat_header_on_new_page: false,
        }
    }
}
