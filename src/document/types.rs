//! PDF Document Types.
//!
//! Structures that represent different Dictionary objects in a PDF document.
mod font;
pub use font::*;
mod actions;
pub use actions::*;
use tux_pdf_low::{
    dictionary,
    types::{Dictionary, Object, ObjectId, ReferenceOrObject},
};

use super::{PageLayout, PageMode};
pub trait PdfType {
    fn into_object(self) -> Object;
}
pub trait PdfDirectoryType {
    fn dictionary_type_key() -> &'static str;
    fn into_dictionary(self) -> Dictionary;

    fn write_to_directory(self, dict: &mut Dictionary)
    where
        Self: Sized,
    {
        let dictionary = self.into_dictionary();
        for (key, value) in dictionary.into_iter() {
            dict.set(key, value.clone());
        }
    }
    #[inline(always)]
    fn into_object(self) -> Object
    where
        Self: Sized,
    {
        self.into_dictionary().into()
    }
}
impl<T> PdfType for T
where
    T: PdfDirectoryType,
{
    fn into_object(self) -> Object {
        self.into_dictionary().into()
    }
}
/// Pages object
///
/// 7.7.3.2
#[derive(Debug, Clone, PartialEq)]
pub struct PagesObject {
    /// An an array of direct references to the page objects in the document
    ///
    /// Includes Count key that indicates the number of pages in the document
    pub kids: Vec<ObjectId>,
    pub resources: Option<ObjectId>,
}
impl PdfDirectoryType for PagesObject {
    fn dictionary_type_key() -> &'static str {
        "Pages"
    }
    fn into_dictionary(self) -> Dictionary {
        let PagesObject { kids, resources } = self;
        let number_of_pages = kids.len() as i64;
        let kids: Vec<_> = kids.into_iter().map(Object::from).collect();
        let mut dict = Dictionary::new();
        dict.set("Type", Object::name(Self::dictionary_type_key()));
        dict.set("Kids", kids);
        dict.set("Count", number_of_pages);
        if let Some(resources) = resources {
            dict.set("Resources", resources);
        }
        dict
    }
}
/// Table 28 – Entries in a page object
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CatalogObject {
    /// /Pages dictionary object id
    pub pages: ObjectId,
    //// PageLayout
    pub page_layout: PageLayout,
    /// PageMode
    pub page_mode: PageMode,
    /// Lang
    pub language: Option<String>,

    pub oc_properties: Option<OptionalContentProperties>,

    pub open_action: Option<PdfAction>,
}
impl PdfDirectoryType for CatalogObject {
    fn dictionary_type_key() -> &'static str {
        "Catalog"
    }
    fn into_dictionary(self) -> Dictionary {
        let CatalogObject {
            pages,
            page_layout: layout,
            language,
            page_mode,
            oc_properties,
            open_action,
        } = self;
        let mut catalog: Dictionary = dictionary! {
            "Type" => Object::name(Self::dictionary_type_key()),
            "Pages" => pages,
            "PageLayout" => layout,
            "PageMode" => page_mode
        };
        if let Some(oc_properties) = oc_properties {
            catalog.set("OCProperties", oc_properties.into_dictionary());
        }
        if let Some(language) = language {
            catalog.set("Lang", Object::string_literal_owned(language));
        }
        if let Some(open_action) = open_action {
            catalog.set("OpenAction", open_action);
        }

        catalog
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Resources {
    pub font: Option<ReferenceOrObject<Dictionary>>,
    pub xobject: Option<ReferenceOrObject<Dictionary>>,
    pub properties: Option<Dictionary>,
}
impl PdfDirectoryType for Resources {
    fn dictionary_type_key() -> &'static str {
        "Resources"
    }
    fn into_dictionary(self) -> Dictionary {
        let Resources {
            font,
            xobject,
            properties,
        } = self;
        let mut dict = Dictionary::new();
        if let Some(font) = font {
            dict.set("Font", font);
        }
        if let Some(xobject) = xobject {
            dict.set("XObject", xobject);
        }
        if let Some(properties) = properties {
            dict.set("Properties", properties);
        }
        dict
    }
}

/// Page object
///
/// 7.7.3.3 Page Tree
pub struct Page {
    /// An array of content stream object ids
    ///
    /// If the array is only 1 it will be written as a single object
    pub contents_id: Vec<ObjectId>,
    pub parent_id: ObjectId,
    pub resources_id: Option<ObjectId>,
    pub media_box: Vec<Object>,
    pub crop_box: Option<Vec<Object>>,
    pub art_box: Option<Vec<Object>>,
    pub bleed_box: Option<Vec<Object>>,
    pub trim_box: Option<Vec<Object>>,
    pub rotation: Option<Object>,
}
impl PdfDirectoryType for Page {
    fn dictionary_type_key() -> &'static str {
        "Page"
    }

    fn into_dictionary(self) -> Dictionary {
        let Page {
            contents_id,
            parent_id,
            resources_id,
            media_box,
            crop_box,
            art_box,
            bleed_box,
            trim_box,
            rotation,
        } = self;
        let mut dictionary = dictionary! {
            "Type" => Object::name("Page"),
            "Parent" => parent_id,
            "MediaBox" => media_box,
            "Contents" => Object::maybe_array(contents_id)
        };

        if let Some(resources_id) = resources_id {
            dictionary.set("Resources", resources_id);
        }
        if let Some(crop_box) = crop_box {
            dictionary.set("CropBox", crop_box);
        }
        if let Some(art_box) = art_box {
            dictionary.set("ArtBox", art_box);
        }
        if let Some(bleed_box) = bleed_box {
            dictionary.set("BleedBox", bleed_box);
        }
        if let Some(trim_box) = trim_box {
            dictionary.set("TrimBox", trim_box);
        }
        if let Some(rotation) = rotation {
            dictionary.set("Rotate", rotation);
        }
        dictionary
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OptionalContentGroup {
    //    pub intent: Option<Vec<String>>,
    // TODO Intent and Usage
    pub name: String,
}
impl PdfDirectoryType for OptionalContentGroup {
    fn dictionary_type_key() -> &'static str {
        "OCG"
    }
    fn into_dictionary(self) -> Dictionary {
        let OptionalContentGroup { name } = self;
        let mut dict = Dictionary::new();
        dict.set("Type", Object::name(Self::dictionary_type_key()));
        dict.set("Name", Object::string_literal_owned(name));
        dict
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OptionalContentProperties {
    pub ocgs: Vec<ObjectId>,
    pub d: OptionalContentConfiguration,
}
impl PdfDirectoryType for OptionalContentProperties {
    fn dictionary_type_key() -> &'static str {
        "OCProperties"
    }
    fn into_dictionary(self) -> Dictionary {
        let OptionalContentProperties { ocgs, d } = self;
        let ocgs_array = Object::Array(ocgs.into_iter().map(Object::from).collect());
        let main_config: Dictionary = d.into();
        let mut dict = Dictionary::new();
        dict.set("OCGs", ocgs_array);
        dict.set("D", main_config);
        dict
    }
}
#[derive(Debug, Clone, PartialEq, Default)]

pub struct OptionalContentConfiguration {
    pub ocgs: Vec<ObjectId>,
    pub base_state: Dictionary,
    pub on: Vec<ObjectId>,
    pub off: Vec<ObjectId>,
}

impl From<OptionalContentConfiguration> for Dictionary {
    fn from(_: OptionalContentConfiguration) -> Self {
        Dictionary::new()
    }
}
