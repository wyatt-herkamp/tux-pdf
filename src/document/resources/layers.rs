use core::fmt;
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

use crate::graphics::PdfObject;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct PdfLayerMap {
    pub map: BTreeMap<LayerInternalId, Layer>,
}

#[derive(Debug, PartialEq, Clone, Eq, PartialOrd, Ord)]
pub struct LayerInternalId(pub String);
impl Default for LayerInternalId {
    fn default() -> Self {
        Self(crate::utils::random::random_character_string(32))
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Layer {
    pub name: String,
    pub operations: Vec<PdfObject>,
    pub creator: Option<String>,
    pub intent: LayerIntent,
    pub usage: LayerSubtype,
}

impl Layer {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }
    pub fn add_operation(&mut self, operation: PdfObject) {
        self.operations.push(operation);
    }

    pub fn set_creator(&mut self, creator: impl Into<String>) {
        self.creator = Some(creator.into());
    }
    pub fn set_intent(&mut self, intent: LayerIntent) {
        self.intent = intent;
    }
    pub fn set_usage(&mut self, usage: LayerSubtype) {
        self.usage = usage;
    }
}
#[derive(Debug, PartialEq, Clone, Default)]
pub enum LayerSubtype {
    #[default]
    Artwork,
}
#[derive(Debug, PartialEq, Clone, Default)]
pub enum LayerIntent {
    View,
    #[default]
    Design,
}
impl AsRef<str> for LayerSubtype {
    fn as_ref(&self) -> &str {
        match self {
            LayerSubtype::Artwork => "Artwork",
        }
    }
}
impl Display for LayerSubtype {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
