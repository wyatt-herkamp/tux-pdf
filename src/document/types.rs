//! PDF Document Types.
//!
//! Structures that represent different Dictionary objects in a PDF document.
use lopdf::{dictionary, Dictionary, Object, ObjectId};
mod font;
pub use font::*;

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
}
impl PdfDirectoryType for PagesObject {
    fn dictionary_type_key() -> &'static str {
        "Pages"
    }
    fn into_dictionary(self) -> Dictionary {
        let PagesObject { kids } = self;
        let number_of_pages = kids.len() as i64;
        let kids: Vec<_> = kids.into_iter().map(Object::from).collect();
        let mut dict = lopdf::Dictionary::new();
        dict.set("Type", Self::dictionary_type_key());
        dict.set("Kids", kids);
        dict.set("Count", number_of_pages);
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
        } = self;
        let mut catalog = dictionary! {
            "Type" => Self::dictionary_type_key(),
            "Pages" => pages,
            "PageLayout" => layout,
            "PageMode" => page_mode,
        };

        if let Some(language) = language {
            catalog.set("Lang", language);
        }

        catalog
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Resources {
    pub font: Option<Dictionary>,
}
impl PdfDirectoryType for Resources {
    fn dictionary_type_key() -> &'static str {
        "Resources"
    }
    fn into_dictionary(self) -> Dictionary {
        let Resources { font } = self;
        let mut dict = Dictionary::new();
        if let Some(font) = font {
            dict.set("Font", font);
        }
        dict
    }
}

/// Page object
///
/// 7.7.3.3 Page Tree
pub struct Page {
    pub contents_id: ObjectId,
    pub parent_id: ObjectId,
    pub resources_id: ObjectId,
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
            "Type" => "Page",
            "Parent" => parent_id,
            "Resources" => resources_id,
            "MediaBox" => media_box,
            "Contents" => contents_id,
        };
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
