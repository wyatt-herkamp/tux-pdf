#[cfg(feature = "image")]
mod pdf_image;
#[cfg(feature = "image")]
pub use pdf_image::*;

mod form;
pub use form::*;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct XObjectMap {
    pub map: BTreeMap<XObjectId, XObject>,
}
#[derive(Debug, PartialEq, Clone, Eq, PartialOrd, Ord)]
pub struct XObjectId(pub String);
impl Default for XObjectId {
    fn default() -> Self {
        Self(crate::utils::random::random_character_string(32))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum XObject {
    /// Image XObject, for images
    #[cfg(feature = "image")]
    Image(PdfImage),
    /// Form XObject, NOT A PDF FORM, this just allows repeatable content
    /// on a page
    Form(Box<FormXObject>),
}
