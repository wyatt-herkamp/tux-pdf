pub mod conformance;
mod meta;
mod resources;

use std::io::Write;

use crate::{
    graphics::{OperationWriter, PdfObject, PdfObjectType},
    page::PdfPage,
    TuxPdfError, TuxPdfResult,
};
use ahash::{HashMap, HashMapExt};
use either::Either;
use lopdf::{content::Content, dictionary, Dictionary, Object, ObjectId, Stream};
pub use meta::*;
pub use resources::*;
use types::{OptionalContentProperties, Page, PagesObject, PdfDirectoryType, Resources};
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

    pub fn add_xobject<T>(&mut self, xobject: T) -> XObjectId
    where
        T: Into<XObject>,
    {
        self.resources.xobjects.add_xobject(xobject.into())
    }
    /// Saves the PDF document to a writer
    pub fn save_to<W: Write>(self, writer: &mut W) -> TuxPdfResult<()> {
        let mut document = self.save_to_lopdf_document()?;
        document.save_to(writer)?;
        Ok(())
    }
    /// Saves the PDF document to a [lopdf::Document]
    ///
    /// This is useful if you want to manipulate the document further before saving it to a file
    pub fn save_to_lopdf_document(mut self) -> TuxPdfResult<lopdf::Document> {
        let mut writer = DocumentWriter::default();
        {
            let info_dict: Dictionary = self.metadata.info.into();
            let info_dict_id = writer.insert_object(info_dict.into());
            writer.info_dict = Some(info_dict_id);
        }
        for (layer_id, layer) in self.resources.layers.map.iter() {
            let optional_content_group = layer.create_ocg_dictionary();
            let ocg_id = writer.insert_object(optional_content_group.into_dictionary().into());
            writer.layers.insert(layer_id.clone(), ocg_id);
        }
        for page in self.pages {
            let mut operation_writer = OperationWriter::default();
            for layer in page.layers {
                let Some(layer_content) = self.resources.layers.clone_layer_content(&layer) else {
                    return Err(ResourceNotRegistered::LayerId(layer).into());
                };
                operation_writer.start_layer(layer);
                operations_to_content(&self.resources, layer_content, &mut operation_writer);
                operation_writer.end_layer();
            }
            operations_to_content(&self.resources, page.ops, &mut operation_writer);
            let OperationWriter { operations, layers } = operation_writer;
            let content = Content { operations };
            let content_id =
                writer.insert_object(Stream::new(dictionary! {}, content.encode().unwrap()).into());
            let resources_id = if layers.is_empty() {
                writer.shared_resources_id()
            } else {
                let mut properties = Dictionary::new();
                for layer in layers {
                    properties.set(layer.0.as_str(), writer.layers[&layer]);
                }
                let resources = Resources {
                    font: Some(Either::Left(writer.font_id())),
                    xobject: Some(Either::Left(writer.xobjects_id())),
                    properties: Some(properties),
                };
                writer.insert_object(resources.into_dictionary().into())
            };
            let page = Page {
                parent_id: writer.pages_id(),
                contents_id: content_id,
                resources_id,
                media_box: page.media_box.to_array(),
                crop_box: page.crop_box.map(|cb| cb.to_array()),
                art_box: page.art_box.map(|ab| ab.to_array()),
                bleed_box: page.bleed_box.map(|bb| bb.to_array()),
                trim_box: page.trim_box.map(|tb| tb.to_array()),
                rotation: page.rotate.map(Object::from),
            };

            writer.new_page(page.into_dictionary());
        }

        let fonts = self.resources.fonts.dictionary(&mut writer);

        writer.fonts(fonts);

        let xobjects = self.resources.xobjects.dictionary(&mut writer)?;
        writer.xobjects(xobjects);

        writer.finish()
    }
    pub fn create_layer(&mut self, name: &str) -> LayerId {
        self.resources.layers.create_layer(name)
    }
    pub fn add_page(&mut self, page: PdfPage) {
        self.pages.push(page);
    }
}

fn operations_to_content(
    resources: &PdfResources,
    operations: Vec<PdfObject>,
    writer: &mut OperationWriter,
) {
    for operation in operations {
        operation.write(resources, writer).unwrap();
    }
}
#[derive(Debug, PartialEq, Default, Clone)]
pub struct PageAnnotMap {
    //pub map: BTreeMap<PageAnnotId, PageAnnotation>,
}

pub struct DocumentWriter {
    layers: HashMap<LayerId, ObjectId>,
    fonts: Option<ObjectId>,
    xobjects: Option<ObjectId>,
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
            layers: HashMap::new(),
            fonts: None,
            xobjects: None,
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
        let font_id = self.font_id();
        self.document.set_object(font_id, fonts);
    }
    pub fn xobjects(&mut self, xobjects: Dictionary) {
        let xobjects_id = self.xobjects_id();
        self.document.set_object(xobjects_id, xobjects);
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
    pub fn font_id(&mut self) -> ObjectId {
        if let Some(font_id) = self.fonts {
            font_id
        } else {
            let font_id = self.new_object_id();
            self.fonts = Some(font_id);
            font_id
        }
    }
    pub fn xobjects_id(&mut self) -> ObjectId {
        if let Some(xobjects_id) = self.xobjects {
            xobjects_id
        } else {
            let xobjects_id = self.new_object_id();
            self.xobjects = Some(xobjects_id);
            xobjects_id
        }
    }
    /// Gets or creates a resources object id
    pub fn shared_resources_id(&mut self) -> ObjectId {
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
            xobjects,
            pages,
            pages_id,
            resources_id,
            info_dict,
            catalog_extras,
            mut document,
            layers,
        } = self;
        let pages_id = pages_id.ok_or(TuxPdfError::NoPagesCreated)?;
        document.set_object(pages_id, PagesObject { kids: pages }.into_dictionary());
        if let Some(resources_id) = resources_id {
            let resources = Resources {
                font: fonts.map(Either::Left),
                xobject: xobjects.map(Either::Left),
                ..Default::default()
            };
            document.set_object(resources_id, resources.into_dictionary());
        }

        let mut catalog_object = catalog_extras
            .unwrap_or_default()
            .create_catalog_object(pages_id);
        if !layers.is_empty() {
            let oc_properties = OptionalContentProperties {
                ocgs: layers.into_values().collect(),
                ..Default::default()
            };
            catalog_object.oc_properties = Some(oc_properties);
        }
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
