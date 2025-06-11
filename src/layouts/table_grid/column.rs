use std::borrow::Cow;

use crate::{
    TuxPdfError,
    document::PdfDocument,
    graphics::{
        PartialOrFullTextStyle, TextBlockContent, TextStyle,
        size::{RenderSize, Size},
    },
    layouts::table::ColumnStyle,
};

use super::style::{
    GridColumnRules,
    size::{ColumnMaxWidth, ColumnMinWidth},
};

/// A new Grid Column based on the the size of the column header text
#[derive(Debug, Clone, PartialEq)]
pub struct SizedColumn {
    pub initial_size: Size,
    pub rules: GridColumnRules,
}
/// A new Grid Column based on the the size of the column header text
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnHeader {
    pub header: TextBlockContent,
    pub styles: Option<ColumnStyle>,
}
impl ColumnHeader {
    pub fn new(header: TextBlockContent) -> Self {
        Self {
            header,
            styles: None,
        }
    }
    pub fn with_min_width(mut self, min_width: ColumnMinWidth) -> Self {
        if let Some(styles) = self.styles.as_mut() {
            styles.min_width = Some(min_width);
        } else {
            self.styles = Some(ColumnStyle {
                min_width: Some(min_width),
                ..Default::default()
            });
        }
        self
    }
    pub fn with_max_width(mut self, max_width: ColumnMaxWidth) -> Self {
        if let Some(styles) = self.styles.as_mut() {
            styles.max_width = Some(max_width);
        } else {
            self.styles = Some(ColumnStyle {
                max_width: Some(max_width),
                ..Default::default()
            });
        }
        self
    }
}
impl<T> From<T> for ColumnHeader
where
    T: Into<TextBlockContent>,
{
    fn from(header: T) -> Self {
        Self {
            header: header.into(),
            styles: None,
        }
    }
}
pub(crate) trait HeaderSize {
    fn header_row_text_styles(&self) -> Option<&PartialOrFullTextStyle>;
    fn root_text_styles(&self) -> &TextStyle;

    fn column_iter(&self) -> impl Iterator<Item = Cow<'_, ColumnHeader>>;
    fn header_text_styles(&self) -> Cow<'_, TextStyle> {
        if let Some(header_text_style) = self.header_row_text_styles() {
            header_text_style.merge_with_full(&self.root_text_styles())
        } else {
            Cow::Borrowed(self.root_text_styles())
        }
    }
    fn size_of_header_columns(
        &self,
        document: &mut PdfDocument,
    ) -> Result<Vec<SizedColumn>, TuxPdfError> {
        let style = self.header_text_styles();
        self.column_iter()
            .map(|column| {
                let size = if let Some(column_styles) =
                    column.styles.as_ref().and_then(|s| s.text_style.as_ref())
                {
                    let style = column_styles.merge_with_full(style.as_ref());
                    column.header.render_size(document, style.as_ref())?
                } else {
                    column.header.render_size(document, style.as_ref())?
                };

                let rules = GridColumnRules {
                    min_width: column.styles.as_ref().and_then(|s| s.min_width),
                    max_width: column.styles.as_ref().and_then(|s| s.max_width),
                };
                Ok(SizedColumn {
                    initial_size: size,
                    rules,
                })
            })
            .collect()
    }
}
