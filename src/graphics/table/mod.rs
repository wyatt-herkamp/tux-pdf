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
    layouts::grid::{GridLayout, GridStyleGroup},
    size::{SimpleRenderSize, Size},
    styles::Margin,
    Text, TextStyle,
};
use crate::graphics::layouts::grid::{
    GridColumnRules, GridLayoutBuilder, GridStyles, NewGridColumm,
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
pub type NewPageFn = fn(document: &mut PdfDocument) -> (TablePageRules, PdfPage);
/// Default new page function
fn default_new_page(_: &mut PdfDocument) -> (TablePageRules, PdfPage) {
    let page = PdfPage::new_from_page_size(A4);
    (Default::default(), page)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table<'table> {
    pub columns: Vec<Column<'table>>,
    pub rows: Vec<Row>,
    pub styles: TableStyles,
    pub new_page: NewPageFn,
}
impl Default for Table<'_> {
    fn default() -> Self {
        Self {
            columns: Default::default(),
            rows: Default::default(),
            styles: Default::default(),
            new_page: default_new_page,
        }
    }
}
impl<'table> Table<'table> {
    pub fn add_column(&mut self, column: Column<'table>) {
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
        // TODO: Merge styles
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
    ) -> Result<Vec<NewGridColumm>, TuxPdfError> {
        let style = self.header_text_styles();
        self.columns
            .iter()
            .map(|column| {
                let size = column.header.render_size(document, style.as_ref())?;
                let rules = GridColumnRules {
                    min_width: column.min_width,
                    ..Default::default()
                };
                Ok(NewGridColumm {
                    initial_size: size,
                    rules,
                })
            })
            .collect()
    }
    fn get_base_grid_row_styles_for_index(&self, index: usize) -> GridStyleGroup {
        match &self.styles.row_colors {
            TableRowColoring::Alternating { alternating_styles } => {
                alternating_styles.row_for_index(index).into()
            }
            TableRowColoring::Single { row_style } => row_style.as_ref().into(),
        }
    }
    fn build_pages(
        &mut self,
        document: &mut PdfDocument,
        first_page: (TablePageRules, PdfPage),
    ) -> Result<Vec<InternalTablePage>, TuxPdfError> {
        let mut pages = Vec::with_capacity(1);
        let mut page = first_page.1;
        // Initialize the first grid builder
        let (columns, header_row_grid_styles) = {
            let column_sizes = self.size_of_header_columns(document)?;
            let mut header_row_styles = self.get_base_grid_row_styles_for_index(0);
            header_row_styles
                .merge_with_option(self.styles.header_styles.as_ref().map(|s| s.into()));
            (column_sizes, Some(header_row_styles))
        };

        let mut grid_builder = GridLayoutBuilder::new(
            &first_page.0,
            self.styles.grid_styles.clone(),
            columns,
            header_row_grid_styles,
        )?;

        info!(?grid_builder);
        let mut rows = Vec::with_capacity(5);

        for (index, row) in mem::take(&mut self.rows).into_iter().enumerate() {
            let column_sizes = row.calculate_sizes(document, &self.styles.text_styles)?;
            let mut grid_styling: GridStyleGroup = self.get_base_grid_row_styles_for_index(index);
            grid_styling.merge_with_option(row.styles.as_ref().map(|s| s.into()));
            debug!(?grid_styling, "Row Styling");

            if !grid_builder.next_row(&column_sizes, Some(grid_styling))? {
                pages.push(InternalTablePage {
                    page,
                    rows: mem::take(&mut rows),
                    grid_layout: grid_builder.build(),
                });
                let (page_rules, new_page) = (self.new_page)(document);

                let (columns, header_row_grid_styles) = {
                    let column_sizes = self.size_of_header_columns(document)?;
                    let mut header_row_styles = self.get_base_grid_row_styles_for_index(0);
                    header_row_styles
                        .merge_with_option(self.styles.header_styles.as_ref().map(|s| s.into()));
                    (column_sizes, Some(header_row_styles))
                };
                grid_builder = GridLayoutBuilder::new(
                    &page_rules,
                    self.styles.grid_styles.clone(),
                    columns,
                    header_row_grid_styles,
                )?;
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
                        .map(|(column, location)| {
                            let value = Cow::Owned(column.header.clone().into_owned());
                            Text {
                                value,
                                position: location,
                                style: header_styles.clone().into_owned(),
                            }
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
                            let value = Cow::Owned(value);

                            let text = Text {
                                value,
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
    grid_layout: GridLayout,
}
