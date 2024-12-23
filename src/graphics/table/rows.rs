use std::borrow::Cow;

use crate::{
    document::PdfDocument,
    graphics::{
        layouts::grid::GridColumnMinWidth,
        size::{SimpleRenderSize, Size},
        TextStyle,
    },
};

use super::{CellStyle, RowStyles};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Column<'column> {
    pub header: Cow<'column, str>,
    /// The minimum width of the column
    pub min_width: Option<GridColumnMinWidth>,
    pub styles: Option<CellStyle>,
}
impl Column<'_> {
    pub fn with_styles(mut self, styles: CellStyle) -> Self {
        self.styles = Some(styles);
        self
    }
    pub fn with_min_width(mut self, min_width: GridColumnMinWidth) -> Self {
        self.min_width = Some(min_width);
        self
    }
}

impl<'column> From<&'column str> for Column<'column> {
    fn from(header: &'column str) -> Self {
        Self {
            header: Cow::Borrowed(header),
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

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub values: Vec<TableValueWithStyle>,
    /// Override the default styles for this row
    pub styles: Option<RowStyles>,
}
impl Row {
    pub fn with_styles(mut self, styles: RowStyles) -> Self {
        self.styles = Some(styles);
        self
    }
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
