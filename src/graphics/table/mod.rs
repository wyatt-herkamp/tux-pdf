use std::{borrow::Cow, mem};

use crate::{
    document::PdfDocument,
    page::{page_sizes::A4, PdfPage},
    units::Pt,
    utils::Merge,
    TuxPdfError,
};
mod style;
pub use style::*;
mod rows;
use super::{
    layouts::table::{GridStyleGroup, TableLayout},
    size::{RenderSize, Size},
    styles::Margin,
    TextBlock, TextStyle,
};
use crate::graphics::layouts::table::{
    GridColumnRules, GridStyles, NewTableColumn, TableLayoutBuilder,
};
pub use rows::*;
use thiserror::Error;
use tracing::{debug, info, Level};

#[derive(Debug, Error)]
pub enum TableError {
    #[error("Row is too wide for the table")]
    RowTooWide,
    #[error("Number of Columns and Values do not match expected {columns} got {values}")]
    ColumnValueMismatch {
        columns: usize,
        values: usize,
        in_row: Option<usize>,
    },
    #[error("Header row is too wide for the table")]
    HeaderDoesNotFit,
    #[error("Columns have not been initialized")]
    GridBuilderColumnsNotInitialized,
    #[error("A Grid can only have at max 1 autofill columns")]
    MultipleAutoFillColumns,

    #[error("Table is not allowed to create more pages")]
    NoNewPageAllowed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TablePageRules {
    pub page_size: Size,
    pub table_start_y: Option<Pt>,
    pub table_stop_y: Option<Pt>,
    /// The margin for the page Left, Right
    pub margin: Option<Margin>,
}
impl Default for TablePageRules {
    fn default() -> Self {
        Self {
            page_size: A4,
            table_start_y: None,
            table_stop_y: None,
            margin: None,
        }
    }
}
pub type NewPageFn =
    fn(document: &mut PdfDocument) -> Result<(TablePageRules, PdfPage), TuxPdfError>;

pub fn no_new_page_allowed(_: &mut PdfDocument) -> Result<(TablePageRules, PdfPage), TuxPdfError> {
    Err(TableError::NoNewPageAllowed.into())
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
    pub styles: TableStyles,
    pub new_page: NewPageFn,
}
impl Default for Table {
    fn default() -> Self {
        Self {
            columns: Default::default(),
            rows: Default::default(),
            styles: Default::default(),
            new_page: no_new_page_allowed,
        }
    }
}
impl Table {
    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }
    pub fn add_row(&mut self, row: Row) {
        self.rows.push(row);
    }
    pub fn number_of_columns(&self) -> usize {
        self.columns.len()
    }
    pub fn number_of_rows(&self) -> usize {
        self.rows.len()
    }
    pub fn validate(&self) -> Result<(), TableError> {
        let columns = self.number_of_columns();
        for (row_index, row) in self.rows.iter().enumerate() {
            let values = row.number_of_columns();
            if columns != values {
                return Err(TableError::ColumnValueMismatch {
                    columns,
                    values,
                    in_row: Some(row_index),
                });
            }
        }
        Ok(())
    }

    fn header_text_styles(&self) -> Cow<'_, TextStyle> {
        if let Some(header_text_style) = self
            .styles
            .header_styles
            .as_ref()
            .and_then(|style| style.text_style.as_ref())
        {
            header_text_style.merge_with_full(&self.styles.text_styles)
        } else {
            Cow::Borrowed(&self.styles.text_styles)
        }
    }
    fn size_of_header_columns(
        &self,
        document: &mut PdfDocument,
    ) -> Result<Vec<NewTableColumn>, TuxPdfError> {
        let style = self.header_text_styles();
        self.columns
            .iter()
            .map(|column| {
                let size = column.header.render_size(document, style.as_ref())?;
                let rules = GridColumnRules {
                    min_width: column.styles.as_ref().and_then(|s| s.min_width),
                    max_width: column.styles.as_ref().and_then(|s| s.max_width),
                };
                Ok(NewTableColumn {
                    initial_size: size,
                    rules,
                })
            })
            .collect()
    }
    fn prepare_content(
        &mut self,
        document: &mut PdfDocument,
        available_size: Size,
    ) -> Result<(), TuxPdfError> {
        let header_text_styles: TextStyle = self.header_text_styles().into_owned();

        for (column_index, column) in self.columns.iter_mut().enumerate() {
            if let Some(max_width) = column.styles.as_ref().and_then(|s| s.max_width) {
                let max_width = match max_width {
                    super::layouts::table::TableColumnMaxWidth::Fixed(pt) => pt,
                    super::layouts::table::TableColumnMaxWidth::Percentage(percentage) => {
                        available_size.width * percentage
                    }
                };
                column
                    .header
                    .apply_max_width(max_width, document, &header_text_styles)?;

                for row in self.rows.iter_mut() {
                    let value = row.values.get_mut(column_index).unwrap();
                    #[allow(clippy::single_match)]
                    match &mut value.value {
                        TableValue::Text(text) => {
                            text.apply_max_width(max_width, document, &self.styles.text_styles)?;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
    fn build_pages(
        &mut self,
        document: &mut PdfDocument,
        first_page: (TablePageRules, PdfPage),
    ) -> Result<Vec<InternalTablePage>, TuxPdfError> {
        let mut pages = Vec::with_capacity(1);
        let mut page = first_page.1;
        let header_row_as_column_group: Option<GridStyleGroup> =
            self.styles.header_styles.as_ref().map(|s| s.into());
        let header_row_styles = self
            .styles
            .row_styles
            .merge_with_option_into_new(header_row_as_column_group);
        let grid_styles = GridStyles {
            cell_content_padding: self.styles.cell_content_padding,
            outer_styles: self.styles.outer_styles.clone(),
            row_styles: None,
            cell_styles: self.styles.cell_styles.clone(),
        };
        // Initialize the first grid builder
        let column_sizes = self.size_of_header_columns(document)?;

        let mut grid_builder = TableLayoutBuilder::new(
            &first_page.0,
            grid_styles.clone(),
            column_sizes,
            Some(header_row_styles.clone()),
        )?;

        self.prepare_content(document, grid_builder.available_size())?;
        info!(?grid_builder);
        let mut rows = Vec::with_capacity(5);

        for row in mem::take(&mut self.rows).into_iter() {
            let column_sizes = row.calculate_sizes(document, &self.styles.text_styles)?;
            let grid_styling: GridStyleGroup = self
                .styles
                .row_styles
                .merge_with_option_into_new(row.grid_row_styles());

            debug!(?grid_styling, "Row Styling");

            if !grid_builder.next_row(&column_sizes, Some(grid_styling.clone()))? {
                pages.push(InternalTablePage {
                    page,
                    rows: mem::take(&mut rows),
                    grid_layout: grid_builder.build(),
                });
                let (page_rules, new_page) = (self.new_page)(document)?;

                let header_column_sizes = self.size_of_header_columns(document)?;

                grid_builder = TableLayoutBuilder::new(
                    &page_rules,
                    grid_styles.clone(),
                    header_column_sizes,
                    Some(header_row_styles.clone()),
                )?;
                grid_builder.next_row(&column_sizes, Some(grid_styling))?;
                page = new_page;
            }
            rows.push(row);
        }
        let grid = grid_builder.build();

        pages.push(InternalTablePage {
            page,
            rows,
            grid_layout: grid,
        });
        Ok(pages)
    }
    pub fn render(
        mut self,
        document: &mut PdfDocument,
        first_page: (TablePageRules, PdfPage),
    ) -> Result<(), TuxPdfError> {
        self.validate()?;
        let pages = self.build_pages(document, first_page)?;
        for table_page in pages {
            let InternalTablePage {
                mut page,
                rows,
                grid_layout,
            } = table_page;
            // Todo: Use actual styles
            let graphics_items = grid_layout.table_graphics();
            if tracing::enabled!(Level::TRACE) {
                tracing::trace!(?graphics_items);
            }
            page.add_operation(graphics_items.into());
            let mut row_iter = grid_layout.row_iter();
            {
                // Render head row
                let header_row_locations = row_iter.next().unwrap();
                let header_styles = self.header_text_styles();

                for text in
                    self.columns
                        .iter()
                        .zip(header_row_locations)
                        .map(|(column, location)| TextBlock {
                            content: column.header.clone(),
                            position: location,
                            style: header_styles.clone().into_owned(),
                        })
                {
                    page.add_operation(text.into());
                }
            }
            for (row, locations) in rows.into_iter().zip(row_iter) {
                let row_text_style = if let Some(styles) = row.styles.and_then(|s| s.text_style) {
                    styles
                        .merge_with_full(&self.styles.text_styles)
                        .into_owned()
                } else {
                    self.styles.text_styles.clone()
                };
                for (column, location) in row.values.into_iter().zip(locations) {
                    #[allow(clippy::single_match)]
                    match column.value {
                        TableValue::Text(value) => {
                            let text = TextBlock {
                                content: value,
                                position: location,
                                style: row_text_style.clone(),
                            };
                            page.add_operation(text.into());
                        }
                        _ => {}
                    }
                }
            }
            document.add_page(page);
        }

        Ok(())
    }
}

struct InternalTablePage {
    page: PdfPage,
    rows: Vec<Row>,
    grid_layout: TableLayout,
}
