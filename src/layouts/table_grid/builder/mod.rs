mod builder_internal;

pub use builder_internal::*;
use tracing::{debug, error};

use crate::{
    graphics::{
        GraphicItems, GraphicStyles, GraphicsGroup, PdfPosition,
        primitives::StraightLine,
        shapes::{OutlineRect, PaintedRect, RectangleStyleType},
        size::Size,
    },
    layouts::table_grid::style::{GridCell, GridStyles},
    utils::Merge,
};

use super::row::SizedGridRow;
pub struct TableRowPlacementIter<'a> {
    table_rect: &'a TableLayout,
    current_row: usize,
}
impl<'a> Iterator for TableRowPlacementIter<'a> {
    type Item = TableColumnPlacementIter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row >= self.table_rect.rows.len() {
            return None;
        }
        let column_locations = TableColumnPlacementIter {
            table_rect: self.table_rect,
            row: self.current_row,
            current_column: 0,
        };
        self.current_row += 1;
        Some(column_locations)
    }
}
pub struct TableColumnPlacementIter<'a> {
    table_rect: &'a TableLayout,
    row: usize,
    current_column: usize,
}
impl Iterator for TableColumnPlacementIter<'_> {
    type Item = PdfPosition;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_column >= self.table_rect.columns.len() {
            return None;
        }
        let column = &self.table_rect.columns[self.current_column];
        let row = &self.table_rect.rows[self.row];
        let result = PdfPosition {
            x: column.content_x,
            y: row.content_y,
        };

        self.current_column += 1;
        Some(result)
    }
}

/// A per table representation of the table
///
/// A new one these must be created for each page
#[derive(Debug, Clone, PartialEq)]
pub struct TableLayout {
    pub(crate) final_size: Size,
    pub(crate) start: PdfPosition,
    pub(crate) styles: GridStyles,
    pub(crate) rows: Vec<SizedGridRow>,
    pub(crate) columns: Vec<GridCell>,
}
impl TableLayout {
    /// Gets the column location for a row and column
    pub fn get_cell_location(&self, row: usize, column: usize) -> Option<PdfPosition> {
        if row >= self.rows.len() {
            error!("Row {} is out of bounds", row);
            return None;
        }
        if column >= self.columns.len() {
            error!("Column {} is out of bounds", column);
            return None;
        }
        let x = self.columns[column].content_x;
        let y = self.rows[row].content_y;

        Some(PdfPosition { x, y })
    }

    pub fn row_iter(&self) -> TableRowPlacementIter {
        TableRowPlacementIter {
            table_rect: self,
            current_row: 0,
        }
    }
    fn table_outline(&self) -> Option<GraphicItems> {
        let outer_styles = self.styles.outer_styles.clone()?;
        let styles: GraphicStyles = (&outer_styles).into();

        if styles.fill_color.is_none() {
            let position = self.start;
            debug!(?styles, "Table Outline");
            let simple_lines: StraightLine = OutlineRect {
                position,
                size: self.final_size,
            }
            .into();

            return Some(
                GraphicsGroup {
                    styles: Some(styles),
                    items: vec![simple_lines.into()],
                    ..Default::default()
                }
                .into(),
            );
        }
        let paint_mode = outer_styles.paint_mode()?;

        let position = PdfPosition {
            x: self.start.x,
            y: self.start.y - self.final_size.height,
        };
        debug!(?styles, ?position, "Table Solid");

        let row_rect = PaintedRect {
            position,
            size: self.final_size,
            paint_mode,
            ..Default::default()
        };
        debug!(?row_rect, "Rendering Table Outline");
        Some(
            GraphicsGroup {
                styles: Some(styles),
                items: vec![row_rect.into()],
                ..Default::default()
            }
            .into(),
        )
    }

    fn rows(&self) -> Vec<GraphicItems> {
        let mut rows = Vec::new();
        let row_styles = self.styles.row_styles.clone();
        for row in &self.rows {
            let row_height = row.height;
            let paint_mode = row
                .styles
                .as_ref()
                .and_then(|s| s.paint_mode())
                .unwrap_or_default();
            let row_styles = match (&row_styles, &row.styles) {
                (Some(row_styles), Some(cell_styles)) => {
                    let mut row_styles = row_styles.clone();
                    row_styles.merge(cell_styles.clone());
                    row_styles
                }
                (Some(row_styles), None) => row_styles.clone(),
                (None, Some(cell_styles)) => cell_styles.clone(),
                (None, None) => continue,
            };
            let position = PdfPosition {
                x: self.start.x,
                y: row.border_line_y,
            };
            let size = Size {
                width: self.final_size.width,
                height: row_height,
            };
            let row_rect = PaintedRect {
                position,
                size,
                paint_mode,
                ..Default::default()
            };
            debug!(?row_rect, "Rendering Row");
            rows.push(
                GraphicsGroup {
                    styles: Some(row_styles.into()),
                    items: vec![row_rect.into()],
                    ..Default::default()
                }
                .into(),
            );
        }
        rows
    }

    fn cells(&self) -> Vec<GraphicItems> {
        let Some(_cell_styles) = self.styles.cell_styles.clone() else {
            return vec![];
        };
        let mut cells = Vec::new();
        for row in &self.rows {
            for column in &self.columns {
                let cell_box_position = PdfPosition {
                    x: column.x,
                    y: row.border_line_y + row.height,
                };
                let cell_box_size = Size {
                    width: column.width,
                    height: row.height,
                };

                let cell_box: OutlineRect = OutlineRect {
                    position: cell_box_position,
                    size: cell_box_size,
                };
                let cell_box: GraphicItems = cell_box.into();
                cells.push(cell_box);
            }
        }
        cells
    }
    /// Creates the Graphics to render the table
    ///
    /// Such as Backgrounds and borders, and other lines
    pub fn table_graphics(&self) -> GraphicsGroup {
        let mut lines: Vec<GraphicItems> = self.rows();
        lines.extend(self.cells());
        lines.extend(self.table_outline());

        GraphicsGroup {
            styles: None,
            items: lines,
            ..Default::default()
        }
    }
}
