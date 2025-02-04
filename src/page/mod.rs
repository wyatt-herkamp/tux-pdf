use crate::{
    document::LayerId,
    graphics::{shapes::OutlineRect, size::Size, LayerType, PdfObject},
};

pub mod page_sizes;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PdfPage {
    pub media_box: OutlineRect,
    pub art_box: Option<OutlineRect>,
    pub bleed_box: Option<OutlineRect>,
    pub trim_box: Option<OutlineRect>,
    pub crop_box: Option<OutlineRect>,
    pub rotate: Option<i64>,
    /// You can think of this as the "content" of the page
    pub contents: Vec<PdfObject>,
    /// Layers that are present on this page
    pub layers: Vec<LayerId>,
}
impl LayerType for PdfPage {
    fn add_to_layer(&mut self, object: impl Into<PdfObject>) -> Result<(), crate::TuxPdfError> {
        self.contents.push(object.into());
        Ok(())
    }
}
impl PdfPage {
    /// Create a new page with the given size
    pub fn new_from_page_size(size: Size) -> Self {
        let media_box = size.into();

        Self {
            media_box,
            contents: Vec::new(),
            layers: Vec::new(),
            ..Default::default()
        }
    }
    pub fn with_crop_box(mut self, crop_box: OutlineRect) -> Self {
        self.crop_box = Some(crop_box);
        self
    }
    pub fn with_art_box(mut self, art_box: OutlineRect) -> Self {
        self.art_box = Some(art_box);
        self
    }
    pub fn with_bleed_box(mut self, bleed_box: OutlineRect) -> Self {
        self.bleed_box = Some(bleed_box);
        self
    }
    pub fn with_trim_box(mut self, trim_box: OutlineRect) -> Self {
        self.trim_box = Some(trim_box);
        self
    }
    /// Add a layer to the page
    pub fn add_layer(&mut self, layer: LayerId) {
        self.layers.push(layer);
    }
}
