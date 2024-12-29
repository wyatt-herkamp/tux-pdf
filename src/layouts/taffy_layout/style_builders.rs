use taffy::Dimension;

#[derive(Debug, Clone)]
pub struct TaffyGridBuilder(taffy::style::Style);
impl Default for TaffyGridBuilder {
    fn default() -> Self {
        Self(taffy::Style {
            display: taffy::Display::Grid,
            ..Default::default()
        })
    }
}
impl From<TaffyGridBuilder> for taffy::style::Style {
    fn from(builder: TaffyGridBuilder) -> Self {
        builder.0
    }
}

impl TaffyGridBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_min_size(mut self, min_size: impl Into<taffy::Size<Dimension>>) -> Self {
        self.0.min_size = min_size.into();
        self
    }

    pub fn with_max_size(mut self, max_size: impl Into<taffy::Size<Dimension>>) -> Self {
        self.0.max_size = max_size.into();
        self
    }
}
