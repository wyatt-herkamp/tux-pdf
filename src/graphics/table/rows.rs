use std::borrow::Cow;

use crate::{
    document::PdfDocument,
    graphics::{
        layouts::grid::{GridColumnMinWidth, GridStyleGroup},
        size::{SimpleRenderSize, Size},
        TextStyle,
    },
};

use super::{CellStyle, ColumnStyle, RowStyles};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Column {
    pub header: String,
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

impl From<&str> for Column {
    fn from(header: &str) -> Self {
        Self {
            header: header.to_owned(),
            ..Default::default()
        }
    }
}
impl From<String> for Column {
    fn from(header: String) -> Self {
        Self {
            header,
            ..Default::default()
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum TableValue {
    Text(String),
    BlankSpace,
}
#[derive(Debug, Clone, PartialEq)]
pub struct TableValueWithStyle {
    pub value: TableValue,
    pub style: Option<CellStyle>,
}
impl From<&str> for TableValue {
    fn from(text: &str) -> Self {
        Self::Text(text.to_owned())
    }
}
impl From<String> for TableValue {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}
impl From<()> for TableValue {
    fn from(_: ()) -> Self {
        Self::BlankSpace
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
            .map(|value| {
                let text_style = if let Some(style) = value
                    .style
                    .as_ref()
                    .and_then(|style| style.text_style.as_ref())
                {
                    style.merge_with_full(default_text_style)
                } else {
                    Cow::Borrowed(default_text_style)
                };

                match &value.value {
                    TableValue::Text(text) => (*text).render_size(document, text_style.as_ref()),
                    TableValue::BlankSpace => ().render_size(document, &()),
                }
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
