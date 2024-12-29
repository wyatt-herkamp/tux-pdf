use core::fmt;
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

use crate::{
    document::types::OptionalContentGroup,
    graphics::{LayerType, PdfObject},
};

use super::{IdType, ObjectMapType};

#[derive(Debug, PartialEq, Default, Clone)]
pub struct PdfLayerMap {
    pub map: BTreeMap<LayerId, Layer>,
}
impl PdfLayerMap {
    pub fn get_layer(&self, id: &LayerId) -> Option<&Layer> {
        self.map.get(id)
    }
    pub fn get_layer_mut(&mut self, id: &LayerId) -> Option<&mut Layer> {
        self.map.get_mut(id)
    }
    pub fn clone_layer_content(&self, id: &LayerId) -> Option<Vec<PdfObject>> {
        self.map.get(id).map(|layer| layer.operations.clone())
    }

    pub fn create_layer(&mut self, name: &str) -> LayerId {
        let id = LayerId::new_random();
        self.map.insert(id.clone(), Layer::new(name));
        id
    }
}
impl ObjectMapType for PdfLayerMap {
    type IdType = LayerId;

    fn has_id(&self, id: &Self::IdType) -> bool {
        self.map.contains_key(id)
    }
}
#[derive(Debug, PartialEq, Clone, Eq, PartialOrd, Ord, Hash)]
pub struct LayerId(pub(crate) String);
impl IdType for LayerId {
    fn new_random() -> Self {
        Self(crate::utils::random::random_character_string(32))
    }

    fn as_str(&self) -> &str {
        &self.0
    }

    fn into_string(self) -> String {
        self.0
    }

    fn resource_category(&self) -> &'static str {
        "Layer"
    }
}
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Layer {
    pub name: String,
    /// Please note that the layer operations are not shared between pages.
    /// And has to be cloned for each page.
    pub operations: Vec<PdfObject>,
    pub creator: Option<String>,
    pub intent: LayerIntent,
    pub usage: LayerSubtype,
}
impl LayerType for Layer {
    fn add_to_layer(&mut self, object: PdfObject) -> Result<(), crate::TuxPdfError> {
        self.operations.push(object);
        Ok(())
    }
}
impl Layer {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
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
    pub(crate) fn create_ocg_dictionary(&self) -> OptionalContentGroup {
        OptionalContentGroup {
            name: self.name.clone(),
        }
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
