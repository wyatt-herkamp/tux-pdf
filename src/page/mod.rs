use crate::{
    document::LayerInternalId,
    graphics::{high::OutlineRect, size::Size, PdfOperation},
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
    pub ops: Vec<PdfOperation>,
    /// Layers that are present on this page
    pub layers: Vec<LayerInternalId>,
}

impl PdfPage {
    /// Create a new page with the given size
    pub fn new_from_page_size(size: Size) -> Self {
        let media_box = size.into();

        Self {
            media_box,
            ops: Vec::new(),
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

    /// Add an operation to the page
    pub fn add_operation(&mut self, operation: PdfOperation) {
        self.ops.push(operation);
    }
    /// Add a layer to the page
    pub fn add_layer(&mut self, layer: LayerInternalId) {
        self.layers.push(layer);
    }
}
