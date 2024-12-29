use crate::{layouts::LayoutItemType, TuxPdfError};

use super::{size::Size, HasPosition, PdfPosition};

/// Blank space is good if you wanna leave a "text space" or writing space for later
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlankSpace {
    pub min_size: Option<Size>,
    pub max_size: Option<Size>,
    pub set_size: Option<Size>,
    pub position: Option<PdfPosition>,
}

impl HasPosition for BlankSpace {
    fn position(&self) -> PdfPosition {
        self.position.unwrap_or_default()
    }

    fn set_position(&mut self, position: PdfPosition) {
        self.position = Some(position);
    }
}

impl LayoutItemType for BlankSpace {
    fn calculate_size(
        &mut self,
        _document: &crate::document::PdfDocument,
    ) -> Result<Size, TuxPdfError> {
        if let Some(size) = self.set_size {
            Ok(size)
        } else if let Some(size) = self.min_size {
            Ok(size)
        } else {
            Ok(Size::default())
        }
    }

    fn render<L: super::LayerType>(
        self,
        _: &crate::document::PdfDocument,
        _: &mut L,
    ) -> Result<(), TuxPdfError>
    where
        Self: Sized,
    {
        Ok(())
    }
}
