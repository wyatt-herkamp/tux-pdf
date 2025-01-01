mod pdf_image;
pub use pdf_image::*;

mod form;
pub use form::*;
use std::collections::BTreeMap;
use tux_pdf_low::types::{Dictionary, Object};

use crate::{document::DocumentWriter, TuxPdfError};

use super::{IdType, ObjectMapType};
#[derive(Debug, PartialEq, Clone, Eq, PartialOrd, Ord)]
pub struct XObjectId(pub(crate) String);
impl IdType for XObjectId {
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
        "XObject"
    }
}
#[derive(Debug, PartialEq, Default, Clone)]
pub struct XObjectMap {
    pub map: BTreeMap<XObjectId, XObject>,
}
impl XObjectMap {
    /// Adds an XObject to the map and returns the id
    pub fn add_xobject(&mut self, xobject: XObject) -> XObjectId {
        let id = XObjectId::new_random();
        self.map.insert(id.clone(), xobject);
        id
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn get_xobject<'resources>(
        &'resources self,
        id: &XObjectId,
    ) -> Option<XObjectRef<'resources>> {
        self.map.get(id).map(|xobject| xobject.as_ref())
    }
    pub fn dictionary(&mut self, writer: &mut DocumentWriter) -> Result<Dictionary, TuxPdfError> {
        let mut xobject_dict = Dictionary::new();
        for (id, xobject) in &mut self.map {
            let dictionary: Object = match xobject {
                XObject::Image(image) => image.image.into_stream()?.into(),
                XObject::Form(_) => {
                    todo!("FormXObject dictionary")
                }
            };
            let object_id = writer.insert_object(dictionary);
            xobject_dict.set(id.to_string(), object_id);
        }
        Ok(xobject_dict)
    }
}

impl ObjectMapType for XObjectMap {
    type IdType = XObjectId;

    fn has_id(&self, id: &Self::IdType) -> bool {
        self.map.contains_key(id)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum XObject {
    /// Image XObject, for images
    Image(Box<PdfXObjectImage>),
    /// Form XObject, NOT A PDF FORM, this just allows repeatable content
    /// on a page
    Form(Box<FormXObject>),
}
impl XObject {
    pub fn as_ref(&self) -> XObjectRef {
        match self {
            XObject::Image(image) => XObjectRef::Image(image),
            XObject::Form(form) => XObjectRef::Form(form),
        }
    }
}

impl From<PdfXObjectImage> for XObject {
    fn from(image: PdfXObjectImage) -> Self {
        XObject::Image(Box::new(image))
    }
}
impl From<FormXObject> for XObject {
    fn from(form: FormXObject) -> Self {
        XObject::Form(Box::new(form))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum XObjectRef<'resources> {
    Image(&'resources PdfXObjectImage),
    Form(&'resources FormXObject),
}
