pub mod conformance;
mod meta;
mod resources;

use crate::{
    graphics::{OperationWriter, PdfOperation, PdfOperationType},
    page::PdfPage,
    TuxPdfError, TuxPdfResult,
};
use lopdf::{content::Content, dictionary, Dictionary, Object, ObjectId, Stream};
pub use meta::*;
pub use resources::*;
use types::{Page, PagesObject, PdfDirectoryType, Resources};
pub mod types;
pub struct PdfDocument {
    /// Metadata about the document (author, info, XMP metadata, etc.)
    pub metadata: PdfMetadata,
    /// Resources shared between pages, such as fonts, XObjects, images, forms, ICC profiles, etc.
    pub resources: PdfResources,
    /// Document-level bookmarks (used for the outline)
    pub bookmarks: PageAnnotMap,
    /// Page contents
    pages: Vec<PdfPage>,
}
impl AsRef<PdfResources> for PdfDocument {
    fn as_ref(&self) -> &PdfResources {
        &self.resources
    }
}
impl AsRef<PdfFontMap> for PdfDocument {
    fn as_ref(&self) -> &PdfFontMap {
        &self.resources.fonts
    }
}
impl PdfDocument {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            metadata: PdfMetadata {
                info: PdfDocumentInfo {
                    document_title: name.into(),
                    ..Default::default()
                },
                xmp: None,
            },
            resources: PdfResources::default(),
            bookmarks: PageAnnotMap::default(),
            pages: Vec::new(),
        }
    }
    pub fn font_map(&mut self) -> &mut PdfFontMap {
        &mut self.resources.fonts
    }

    pub fn write_to_lopdf_document(self) -> TuxPdfResult<lopdf::Document> {
        let mut writer = DocumentWriter::default();
        {
            let info_dict: Dictionary = self.metadata.info.into();
            let info_dict_id = writer.insert_object(info_dict.into());
            writer.info_dict = Some(info_dict_id);
        }
        let fonts = self.resources.fonts.dictionary(&mut writer);
        writer.fonts(fonts);

        for page in self.pages {
            let content = operations_to_content(&self.resources, page.ops);
            let content_id =
                writer.insert_object(Stream::new(dictionary! {}, content.encode().unwrap()).into());

            let page = Page {
                parent_id: writer.pages_id(),
                contents_id: content_id,
                resources_id: writer.resources_id(),
                media_box: page.media_box.to_array(),
                crop_box: page.crop_box.map(|cb| cb.to_array()),
                art_box: page.art_box.map(|ab| ab.to_array()),
                bleed_box: page.bleed_box.map(|bb| bb.to_array()),
                trim_box: page.trim_box.map(|tb| tb.to_array()),
                rotation: page.rotate.map(Object::from),
            };

            writer.new_page(page.into_dictionary());
        }

        writer.finish()
    }
    pub fn add_page(&mut self, page: PdfPage) {
        self.pages.push(page);
    }
}

fn operations_to_content(resources: &PdfResources, operations: Vec<PdfOperation>) -> Content {
    let mut writer = OperationWriter::default();
    for operation in operations {
        operation.write(resources, &mut writer).unwrap();
    }
    writer.into()
}
#[derive(Debug, PartialEq, Default, Clone)]
pub struct PageAnnotMap {
    //pub map: BTreeMap<PageAnnotId, PageAnnotation>,
}

pub struct DocumentWriter {
    fonts: Option<Dictionary>,
    pages: Vec<ObjectId>,
    pages_id: Option<ObjectId>,
    resources_id: Option<ObjectId>,
    info_dict: Option<ObjectId>,
    catalog_extras: Option<CatalogInfo>,
    document: lopdf::Document,
}
impl Default for DocumentWriter {
    fn default() -> Self {
        Self {
            fonts: None,
            document: lopdf::Document::with_version("1.7"),
            pages: Vec::new(),
            pages_id: None,
            info_dict: None,
            resources_id: None,
            catalog_extras: None,
        }
    }
}
impl DocumentWriter {
    pub fn fonts(&mut self, fonts: Dictionary) {
        self.fonts = Some(fonts);
    }
    pub fn insert_object(&mut self, object: lopdf::Object) -> lopdf::ObjectId {
        self.document.add_object(object)
    }
    pub fn new_object_id(&mut self) -> ObjectId {
        self.document.new_object_id()
    }
    pub fn new_page(&mut self, page: Dictionary) -> ObjectId {
        let page_id = self.insert_object(page.into());
        self.pages.push(page_id);
        page_id
    }
    /// Gets or creates a pages object id
    pub fn pages_id(&mut self) -> ObjectId {
        if let Some(pages_id) = self.pages_id {
            pages_id
        } else {
            let pages_id = self.new_object_id();
            self.pages_id = Some(pages_id);
            pages_id
        }
    }
    /// Gets or creates a resources object id
    pub fn resources_id(&mut self) -> ObjectId {
        if let Some(resources_id) = self.resources_id {
            resources_id
        } else {
            let resources_id = self.new_object_id();
            self.resources_id = Some(resources_id);
            resources_id
        }
    }
    pub(crate) fn finish(self) -> TuxPdfResult<lopdf::Document> {
        let Self {
            fonts,
            pages,
            pages_id,
            resources_id,
            info_dict,
            catalog_extras,
            mut document,
        } = self;
        let pages_id = pages_id.ok_or(TuxPdfError::NoPagesCreated)?;
        document.set_object(pages_id, PagesObject { kids: pages }.into_dictionary());
        if let Some(resources_id) = resources_id {
            let resources = Resources { font: fonts };
            document.set_object(resources_id, resources.into_dictionary());
        }
        let catalog_object = catalog_extras
            .unwrap_or_default()
            .create_catalog_object(pages_id);
        // Create Catalog object
        let catalog_id = document.add_object(catalog_object.into_dictionary());
        // Point the Root key to the Pages object
        document.trailer.set("Root", catalog_id);
        if let Some(info_dict) = info_dict {
            document.trailer.set("Info", info_dict);
        }

        Ok(document)
    }
}
