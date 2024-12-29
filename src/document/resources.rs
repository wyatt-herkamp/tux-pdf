mod font;
mod icc_profile;
mod layers;
mod xobject;
use std::fmt::Debug;

pub use font::*;
pub use icc_profile::*;
pub use layers::*;
use thiserror::Error;
pub use xobject::*;

#[derive(Debug, PartialEq, Clone, Error)]
pub enum ResourceNotRegistered {
    #[error("Font not registered: {0:?}")]
    FontId(FontId),
    #[error("Builtin font not registered: {0:?}")]
    BuiltinFontNotRegistered(BuiltinFont),
    #[error("XObject not registered: {0:?}")]
    XObjectId(XObjectId),
    #[error("Layer not registered: {0:?}")]
    LayerId(LayerId),
}
impl From<FontRef> for ResourceNotRegistered {
    fn from(font_ref: FontRef) -> Self {
        match font_ref {
            FontRef::External(font_id) => ResourceNotRegistered::FontId(font_id),
            FontRef::Builtin(builtin_font) => {
                ResourceNotRegistered::BuiltinFontNotRegistered(builtin_font)
            }
        }
    }
}

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
impl PdfResources {
    pub fn get_font_type(&self, font_id: &FontRef) -> Option<InternalFontTypes<'_>> {
        self.fonts.internal_font_type(font_id)
    }
}
pub trait IdType: Debug + Send + Sync {
    fn new_random() -> Self
    where
        Self: Sized;

    fn as_str(&self) -> &str;
    fn into_string(self) -> String
    where
        Self: Sized;
    #[inline]
    fn bytes(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
    #[inline]
    fn into_bytes(self) -> Vec<u8>
    where
        Self: Sized,
    {
        self.into_string().into_bytes()
    }
    /// Used for Error messages
    fn resource_category(&self) -> &'static str;
}
pub(crate) trait ObjectMapType {
    type IdType: IdType;
    /// Checks if the object map has the given id
    fn has_id(&self, id: &Self::IdType) -> bool;
    /// Generates a new unique id for the object map
    fn new_id(&self) -> Self::IdType {
        let mut loop_count = 0;
        loop {
            let id = Self::IdType::new_random();
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
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        impl From<$type> for lopdf::Object {
            fn from(id: $type) -> Self {
                lopdf::Object::Name(id.0.into_bytes())
            }
        }
        impl From<&$type> for lopdf::Object {
            fn from(id: &$type) -> Self {
                lopdf::Object::Name(id.0.clone().into_bytes())
            }
        }
        impl From<$type> for String {
            fn from(id: $type) -> Self {
                id.0
            }
        }
        impl super::types::PdfType for $type {
            fn into_object(self) -> lopdf::Object {
                lopdf::Object::Name(self.0.into_bytes())
            }
        }
        impl From<$type> for ResourceNotRegistered {
            fn from(id: $type) -> Self {
                ResourceNotRegistered::$type(id)
            }
        }
    };
}

object_id_type!(FontId);
object_id_type!(XObjectId);
object_id_type!(LayerId);
