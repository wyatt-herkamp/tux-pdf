use crate::{
    TuxPdfError,
    document::PdfDocument,
    graphics::{
        TextBlockContent, TextStyle,
        size::{RenderSize, Size},
    },
    layouts::table_grid::style::GridStyleGroup,
};

use super::{CellStyle, RowStyles};

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
    ) -> Result<Vec<Size>, TuxPdfError> {
        self.values
            .iter()
            .map(|value| match &value.value {
                TableValue::Text(text) => (*text).render_size(document, default_text_style),
                TableValue::BlankSpace => Ok(().render_size(document, &()).unwrap()),
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
