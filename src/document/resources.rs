mod font;
mod icc_profile;
mod layers;
mod xobject;
pub use font::*;
pub use icc_profile::*;
pub use layers::*;
pub use xobject::*;
#[derive(Debug, Default, PartialEq, Clone)]
pub struct PdfResources {
    /// Fonts found in the PDF file, indexed by the sha256 of their contents
    pub fonts: PdfFontMap,
    /// XObjects (forms, images, embedded PDF contents, etc.)
    pub xobjects: XObjectMap,
    /// Map of explicit extended graphics states
    //pub extgstates: ExtendedGraphicsStateMap,
    /// Map of optional content groups
    pub layers: PdfLayerMap,
}

pub(crate) trait ObjectMapType {
    type IdType: PartialEq + Clone + Default;
    /// Checks if the object map has the given id
    fn has_id(&self, id: &Self::IdType) -> bool;
    /// Generates a new unique id for the object map
    fn new_id(&self) -> Self::IdType {
        let mut loop_count = 0;
        loop {
            let id = Self::IdType::default();
            if !self.has_id(&id) {
                break id;
            }
            // Just in case something goes wrong
            loop_count += 1;
            if loop_count > 100 {
                panic!("Failed to generate a unique font id. This should never happen. Like what the heck?");
            }
        }
    }
}

macro_rules! object_id_type {
    (
        $type:ident
    ) => {
        impl std::convert::AsRef<str> for $type {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        impl From<$type> for lopdf::Object {
            fn from(id: $type) -> Self {
                id.0.into()
            }
        }
        impl From<$type> for String {
            fn from(id: $type) -> Self {
                id.0
            }
        }
        impl super::types::PdfType for $type {
            fn into_object(self) -> lopdf::Object {
                self.0.into()
            }
        }
    };
}

object_id_type!(FontId);
