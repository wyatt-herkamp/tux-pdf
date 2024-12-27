use crate::{
    document::PdfDocument,
    graphics::{
        layouts::grid::{GridColumnMaxWidth, GridColumnMinWidth, GridStyleGroup},
        size::{RenderSize, Size},
        TextBlockContent, TextStyle,
    },
};

use super::{CellStyle, ColumnStyle, RowStyles};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Column {
    pub header: TextBlockContent,
    pub styles: Option<ColumnStyle>,
}
impl Column {
    pub fn with_cell_styles(mut self, styles: CellStyle) -> Self {
        if let Some(column_styles) = self.styles.as_mut() {
            column_styles.cell_styles = Some(styles);
        } else {
            self.styles = Some(ColumnStyle {
                cell_styles: Some(styles),
                ..Default::default()
            });
        }

        self
    }
    pub fn with_max_width(mut self, max_width: GridColumnMaxWidth) -> Self {
        if let Some(column_styles) = self.styles.as_mut() {
            column_styles.max_width = Some(max_width);
        } else {
            self.styles = Some(ColumnStyle {
                max_width: Some(max_width),
                ..Default::default()
            });
        }
        self
    }
    pub fn with_min_width(mut self, min_width: GridColumnMinWidth) -> Self {
        if let Some(column_styles) = self.styles.as_mut() {
            column_styles.min_width = Some(min_width);
        } else {
            self.styles = Some(ColumnStyle {
                min_width: Some(min_width),
                ..Default::default()
            });
        }
        self
    }
}

impl<T> From<T> for Column
where
    T: Into<TextBlockContent>,
{
    fn from(header: T) -> Self {
        Self {
            header: header.into(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableValue {
    Text(TextBlockContent),
    BlankSpace,
}
impl<T> From<T> for TableValue
where
    T: Into<TextBlockContent>,
{
    fn from(header: T) -> Self {
        Self::Text(header.into())
    }
}
impl From<()> for TableValue {
    fn from(_: ()) -> Self {
        Self::BlankSpace
    }
}
impl Default for TableValue {
    fn default() -> Self {
        Self::BlankSpace
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TableValueWithStyle {
    pub value: TableValue,
    pub style: Option<CellStyle>,
}

impl<T> From<T> for TableValueWithStyle
where
    T: Into<TableValue>,
{
    fn from(value: T) -> Self {
        Self {
            value: value.into(),
            style: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Row {
    pub values: Vec<TableValueWithStyle>,
    /// Override the default styles for this row
    pub styles: Option<RowStyles>,
}
impl Row {
    /// Adds styles to the row
    pub fn with_styles(mut self, styles: RowStyles) -> Self {
        self.styles = Some(styles);
        self
    }
    /// Get the number of columns in the row
    pub fn number_of_columns(&self) -> usize {
        self.values.len()
    }
    /// Calculate the sizes of the row
    pub fn calculate_sizes(
        &self,
        document: &PdfDocument,
        default_text_style: &TextStyle,
    ) -> Result<Vec<Size>, crate::TuxPdfError> {
        self.values
            .iter()
            .map(|value| match &value.value {
                TableValue::Text(text) => (*text).render_size(document, default_text_style),
                TableValue::BlankSpace => ().render_size(document, &()),
            })
            .collect()
    }
    pub(crate) fn grid_row_styles(&self) -> Option<GridStyleGroup> {
        self.styles.as_ref().map(|styles| styles.into())
    }
}
impl<T> From<Vec<T>> for Row
where
    T: Into<TableValue>,
{
    fn from(values: Vec<T>) -> Self {
        Self {
            values: values
                .into_iter()
                .map(Into::into)
                .map(|value| TableValueWithStyle { value, style: None })
                .collect(),
            styles: None,
        }
    }
}
