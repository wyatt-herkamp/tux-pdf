use tracing::{debug, error, warn};

use super::{types::*, TableLayout};
use crate::{
    graphics::{size::Size, PdfPosition},
    layouts::table::{TableError, TablePageRules},
    units::{Pt, UnitType},
};

/// A per table representation of the table
///
/// A new one these must be created for each page
#[derive(Debug, Clone, PartialEq)]
pub struct TableLayoutBuilder {
    current_y: Pt,
    next_y: Pt,
    current_x: Pt,
    start: PdfPosition,
    max_grid_size: Size,
    styles: GridStyles,
    rows: Vec<GridBuilderRow>,
    columns: Vec<GridBuilderColumn>,
}
impl TableLayoutBuilder {
    pub fn new(
        table_page_rules: &TablePageRules,
        styles: GridStyles,
        columns: Vec<NewTableColumn>,
        header_row_styles: Option<GridStyleGroup>,
    ) -> Result<Self, TableError> {
        let TablePageRules {
            page_size,
            table_start_y,
            table_stop_y,
            margin,
        } = table_page_rules;
        let page_width = page_size.width.pt();
        let page_height = page_size.height.pt();
        let (left, right, _top, _bottom) = margin.unwrap_or_default().into();

        let max_width = page_width - left - right;
        let starting_y = if let Some(header_area) = table_start_y {
            *header_area
        } else {
            page_height
        };
        let max_height = if let Some(footer_area) = table_stop_y {
            *footer_area
        } else {
            0f32.pt()
        };
        let max_grid_size = Size::new(max_width, max_height);

        let mut builder = Self {
            current_y: starting_y,
            next_y: starting_y,
            current_x: left,
            start: PdfPosition {
                x: left,
                y: starting_y,
            },
            max_grid_size,
            styles,
            rows: Default::default(),
            columns: Default::default(),
        };
        debug!(?builder, "Grid Layout created");
        if !builder.initialize_columns(columns)? {
            return Err(TableError::HeaderDoesNotFit);
        }
        builder.rows[0].styles = header_row_styles;
        debug!(?builder, "Columns initialized");

        Ok(builder)
    }

    pub fn available_size(&self) -> Size {
        Size::new(
            self.max_grid_size.width,
            self.current_y - self.max_grid_size.height,
        )
    }
    /// Calculates the initial columns widths and x positions
    fn initialize_columns(&mut self, columns: Vec<NewTableColumn>) -> Result<bool, TableError> {
        let column_sizes = columns
            .iter()
            .map(|column| column.initial_size)
            .collect::<Vec<Size>>();
        self.columns.reserve(columns.len());
        let mut has_auto_fill = false;
        for (index, column) in columns.into_iter().enumerate() {
            let NewTableColumn {
                initial_size,
                rules,
            } = column;
            if rules
                .min_width
                .map(|w| w == TableColumnMinWidth::AutoFill)
                .unwrap_or(false)
            {
                if has_auto_fill {
                    return Err(TableError::MultipleAutoFillColumns);
                }
                has_auto_fill = true;
            }
            let horizontal_padding = self
                .styles
                .cell_content_padding
                .horizontal_value()
                .unwrap_or_default();
            // Calculate the X position of the column
            let x = self.start.x
                + column_sizes
                    .iter()
                    .take(index)
                    .map(|size| size.width + horizontal_padding)
                    .sum::<Pt>();
            let width = initial_size.width + horizontal_padding;
            let column = GridBuilderColumn {
                width,
                x,
                rules: rules.clone(),
            };
            self.columns.push(column);
        }

        self.next_row(&column_sizes, None)
    }
    /// Checks if any columns need to be recalculated
    ///
    /// If so it will recalculate the columns after the column that needs to be recalculated
    fn recaclulate_columns(&mut self, column_sizes: &[Size]) -> Result<(), TableError> {
        if column_sizes.len() != self.columns.len() {
            return Err(TableError::ColumnValueMismatch {
                columns: self.columns.len(),
                values: column_sizes.len(),
                in_row: Some(self.rows.len()),
            });
        }
        let horizontal_padding = self
            .styles
            .cell_content_padding
            .horizontal_value()
            .unwrap_or_default();
        // Find a column where the new size is larger than the current size
        for (index, size) in column_sizes.iter().enumerate() {
            if (self.columns[index].width) < (size.width + horizontal_padding) {
                // Recalculate this column and all columns after it
                self.recalculate_columns_after(index, column_sizes);
                break;
            }
        }
        Ok(())
    }
    /// Recalculates the columns after a certain column
    fn recalculate_columns_after(&mut self, column_index: usize, columns: &[Size]) {
        let mut x = if column_index == 0 {
            self.start.x
        } else {
            let prev_column = &self.columns[column_index - 1];

            prev_column.x + prev_column.width
        };
        let horizontal_padding = self
            .styles
            .cell_content_padding
            .horizontal_value()
            .unwrap_or_default();
        for (table_column, size) in self
            .columns
            .iter_mut()
            .zip(columns.iter())
            .skip(column_index)
        {
            table_column.x = x + horizontal_padding;
            let new_width = (size.width + horizontal_padding).max(table_column.width);
            table_column.width = new_width;
            x += new_width;
        }
    }
    /// Increases the current y position by the row height
    ///
    /// If it would overflow the max height it will return false meaning a new page is needed
    pub fn next_row(
        &mut self,
        column_sizes: &[Size],
        style: Option<GridStyleGroup>,
    ) -> Result<bool, TableError> {
        if self.columns.is_empty() {
            return Err(TableError::GridBuilderColumnsNotInitialized);
        }
        if column_sizes.len() != self.columns.len() && !self.columns.is_empty() {
            error!(?column_sizes, ?self.columns, "Column sizes and columns mismatch");
            return Err(TableError::ColumnValueMismatch {
                columns: self.columns.len(),
                values: column_sizes.len(),
                in_row: Some(self.rows.len()),
            });
        }

        let row_height_base = column_sizes
            .iter()
            .map(|size| size.height)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let Some(row_height_base) = row_height_base else {
            warn!("No row height base");
            return Ok(false);
        };
        let row_height = row_height_base
            + self
                .styles
                .cell_content_padding
                .vertical_value()
                .unwrap_or_default();

        debug!(?row_height, "Next row");
        // Remember 0,0 is the bottom left corner so we subtract the row height
        let next_y = self.current_y - row_height;
        debug!(?next_y, "Next y");
        if next_y < self.max_grid_size.height {
            debug!(?next_y, ?self.max_grid_size, "Next y is less than max height");
            return Ok(false);
        }
        self.current_y = next_y;
        self.recaclulate_columns(column_sizes)?;

        self.rows.push(GridBuilderRow {
            y: self.current_y
                + self
                    .styles
                    .cell_content_padding
                    .vertical_value()
                    .unwrap_or_default(),
            content_start_y: self.current_y,
            height: row_height,
            styles: style,
        });
        Ok(true)
    }

    pub fn calculate_full_width(&self) -> Pt {
        self.columns
            .iter()
            .fold(0f32.pt(), |acc, cell| acc + cell.width)
    }

    pub fn calculate_full_height(&self) -> Pt {
        self.rows
            .iter()
            .fold(0f32.pt(), |acc, cell| acc + cell.height)
    }
    fn calculate_final_column_width_starting_at(&mut self, start_index: usize) {
        let mut x = if start_index == 0 {
            self.start.x
        } else {
            let prev_column = &self.columns[start_index - 1];

            prev_column.x + prev_column.width
        };

        for table_column in self.columns.iter_mut().skip(start_index) {
            table_column.x = x;
            x += table_column.width;
        }
    }
    fn recalculate_column_x_positions(&mut self) {
        let mut x = self.start.x;
        let horizontal_padding = self
            .styles
            .cell_content_padding
            .horizontal_value()
            .unwrap_or_default();
        for table_column in self.columns.iter_mut() {
            table_column.x = x + horizontal_padding;
            x += table_column.width;
        }
    }
    fn calculate_maximum_available_width(&self) -> Pt {
        self.max_grid_size.width - self.columns.iter().map(|column| column.width).sum::<Pt>()
    }
    fn apply_column_width_overrides(&mut self) {
        let mut auto_fill_index = None;
        for (index, column) in self.columns.iter_mut().enumerate() {
            if let Some(width_override) = column.rules.min_width {
                match width_override {
                    TableColumnMinWidth::Fixed(pt) => {
                        if pt < column.width {
                            error!(?pt, ?column, "Fixed width is less than current width. Skipping until we can handle text wrapping");
                            continue;
                        }
                        column.width = pt;
                    }
                    TableColumnMinWidth::Percentage(percentage) => {
                        let new_width = self.max_grid_size.width * percentage;
                        if new_width < column.width {
                            error!(?new_width, ?column, "Percentage width is less than current width. Skipping until we can handle text wrapping");
                            continue;
                        }
                        column.width = new_width;
                    }
                    TableColumnMinWidth::AutoFill => {
                        auto_fill_index = Some(index);
                    }
                }
            }
        }
        self.recalculate_column_x_positions();
        if let Some(auto_fill_index) = auto_fill_index {
            debug!(?auto_fill_index, "Calculating auto fill column width");
            let available_width = self.calculate_maximum_available_width();
            debug!(?available_width, "Available width");
            let auto_fill_column = &mut self.columns[auto_fill_index];
            auto_fill_column.width += available_width;
            debug!(?auto_fill_column, "Auto fill column");
            self.calculate_final_column_width_starting_at(auto_fill_index - 1);
            debug!(?self.columns, "Columns after auto fill");
        }
    }
    pub fn build(mut self) -> TableLayout {
        self.apply_column_width_overrides();
        let final_size = Size::new(self.calculate_full_width(), self.calculate_full_height());

        let mut columns = Vec::with_capacity(self.columns.len());
        let left_padding = self.styles.cell_content_padding.left.unwrap_or_default();
        let horizontal_padding = self
            .styles
            .cell_content_padding
            .horizontal_value()
            .unwrap_or_default();
        let top_padding = self.styles.cell_content_padding.top.unwrap_or_default();
        let vertical_padding = self
            .styles
            .cell_content_padding
            .vertical_value()
            .unwrap_or_default();
        for column in self.columns {
            let result = GridColumn {
                width: column.width,
                width_no_padding: column.width - horizontal_padding,
                x: column.x - horizontal_padding,
                content_x: column.x - left_padding,
                border_line_x: column.x,
            };
            columns.push(result);
        }
        let rows: Vec<TableRow> = self
            .rows
            .into_iter()
            .map(|row| TableRow {
                content_y: row.content_start_y + top_padding,
                border_line_y: row.y - vertical_padding,
                height: row.height,
                styles: row.styles,
            })
            .collect();

        TableLayout {
            final_size,
            start: self.start,
            styles: self.styles,
            rows,
            columns,
        }
    }
}
