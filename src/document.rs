pub mod conformance;
mod meta;
mod resources;

use std::{io::Write, mem};

use crate::{
    graphics::{OperationWriter, PdfObject, PdfObjectType},
    page::PdfPage,
    TuxPdfError, TuxPdfResult,
};
use ahash::{HashMap, HashMapExt};
pub use meta::*;
pub use resources::*;
use tux_pdf_low::{
    document::PdfDocumentWriter,
    types::{Dictionary, Object, ObjectId, ReferenceOrObject},
};
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
                ..Default::default()
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
        let document = self.write_into_pdf_document_writer()?;
        document.save(writer)?;
        Ok(())
    }
    /// Saves the PDF document to a [PdfDocumentWriter]
    ///
    /// This is useful if you want to manipulate the document further before saving it to a file
    pub fn write_into_pdf_document_writer(mut self) -> TuxPdfResult<PdfDocumentWriter> {
        // Note to future developers: This function requires a very specific order of operations.
        // When writing pages they require the XObjects and Fonts to still be in the resources map
        // Layers can be immeidately removed from resources as nothing else will access them from the resources map

        let mut writer = DocumentWriter::default();
        {
            let info_dict: Dictionary = self.metadata.info.into();
            let info_dict_id = writer.insert_object(info_dict.into());
            writer.info_dict = Some(info_dict_id);
        }
        writer.catalog_extras = Some(self.metadata.catalog_info);
        // Take the layers from resources and create the layers in the writer
        // The pages should not access the layers after this point so it should be fine to take them and leave the resources empty
        for (layer_id, layer) in std::mem::take(&mut self.resources.layers.map).into_iter() {
            let optional_content_group = layer.create_ocg_dictionary();
            let ocg_id = writer.insert_object(optional_content_group.into_dictionary().into());

            // Note: The amount of page ops does != the amount of pdf operations.
            // This is because this library is abstracting pdf and each "page op" is usually multiple operations
            let mut operation_writer: OperationWriter =
                OperationWriter::with_capacity(2 + layer.operations.len());

            operation_writer.start_layer(layer_id.clone());
            operations_to_content(&self.resources, layer.operations, &mut operation_writer);
            operation_writer.end_section();

            let stream_content = operation_writer.into_stream(Dictionary::default())?;
            let stream_id = writer.insert_object(stream_content.into());

            writer
                .layers
                .insert(layer_id.clone(), WriterLayer { ocg_id, stream_id });
        }

        for page in self.pages {
            let mut layers = Vec::new();
            for layer_id in page.layers {
                let layer = writer
                    .layers
                    .get(&layer_id)
                    .ok_or_else(|| ResourceNotRegistered::LayerId(layer_id.clone()))?;

                layers.push((layer_id, *layer));
            }

            let mut content_ids = Vec::with_capacity(layers.len() + 1);
            // Check if the page has any content. If it does, write the content to the page
            if !page.contents.is_empty() {
                // Note: The amount of page ops does != the amount of pdf operations.
                // This is because this library is abstracting pdf and each "page op" is usually multiple operations
                let mut operation_writer: OperationWriter =
                    OperationWriter::with_capacity(page.contents.len());

                operations_to_content(&self.resources, page.contents, &mut operation_writer);
                let content_stream = operation_writer.into_stream(Dictionary::default())?;
                let content_id = writer.insert_object(content_stream.into());
                content_ids.push(content_id);
            }

            let resources_id = if layers.is_empty() {
                writer.uses_shared_resources();
                None
            } else {
                let mut properties = Dictionary::new();
                for (layer_id, layer) in layers {
                    content_ids.push(layer.stream_id);
                    properties.set(layer_id.as_str(), layer.ocg_id);
                }
                let resources = Resources {
                    font: Some(ReferenceOrObject::Reference(writer.font_id())),
                    xobject: Some(ReferenceOrObject::Reference(writer.xobjects_id())),
                    properties: Some(properties),
                };
                Some(writer.insert_object(resources.into_dictionary().into()))
            };
            let page = Page {
                parent_id: writer.pages_id(),
                contents_id: content_ids,
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
        // We can consume the rest of the resources as the only parts of the code that needs them now has been converted into pdf operations
        let PdfResources {
            fonts, xobjects, ..
        } = mem::take(&mut self.resources);
        let fonts = fonts.dictionary(&mut writer);

        writer.fonts(fonts);

        let xobjects = xobjects.dictionary(&mut writer)?;
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
#[derive(Debug, Clone, Copy)]
pub(crate) struct WriterLayer {
    ocg_id: ObjectId,
    stream_id: ObjectId,
}
pub(crate) struct DocumentWriter {
    layers: HashMap<LayerId, WriterLayer>,
    fonts: Option<ObjectId>,
    xobjects: Option<ObjectId>,
    pages: Vec<ObjectId>,

    pages_id: Option<ObjectId>,
    uses_shared_resources: bool,
    resources_id: Option<ObjectId>,
    info_dict: Option<ObjectId>,
    catalog_extras: Option<CatalogInfo>,
    document: PdfDocumentWriter,
}
impl Default for DocumentWriter {
    fn default() -> Self {
        Self {
            layers: HashMap::new(),
            fonts: None,
            xobjects: None,
            document: PdfDocumentWriter::default(),
            pages: Vec::new(),
            pages_id: None,
            info_dict: None,
            uses_shared_resources: false,
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
    pub fn insert_object(&mut self, object: Object) -> ObjectId {
        self.document.add_object(object)
    }
    pub fn new_object_id(&mut self) -> ObjectId {
        self.document.next_object_id()
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
    pub fn uses_shared_resources(&mut self) {
        self.uses_shared_resources = true;
    }
    /// Gets or creates a resources object id
    #[allow(dead_code)]
    pub fn shared_resources_id(&mut self) -> ObjectId {
        if let Some(resources_id) = self.resources_id {
            resources_id
        } else {
            let resources_id = self.new_object_id();
            self.resources_id = Some(resources_id);
            resources_id
        }
    }
    pub(crate) fn finish(self) -> TuxPdfResult<PdfDocumentWriter> {
        let Self {
            fonts,
            xobjects,
            pages,
            pages_id,
            uses_shared_resources,
            resources_id,
            info_dict,
            catalog_extras,
            mut document,
            layers,
        } = self;
        let pages_id = pages_id.ok_or(TuxPdfError::NoPagesCreated)?;
        let shared_resources = if uses_shared_resources {
            let resources = Resources {
                font: fonts.map(ReferenceOrObject::Reference),
                xobject: xobjects.map(ReferenceOrObject::Reference),
                ..Default::default()
            };
            if let Some(resources_id) = resources_id {
                document.set_object(resources_id, resources.into_dictionary());
                Some(resources_id)
            } else {
                Some(document.add_object(resources.into_dictionary()))
            }
        } else {
            None
        };
        document.set_object(
            pages_id,
            PagesObject {
                kids: pages,
                resources: shared_resources,
            }
            .into_dictionary(),
        );

        let mut catalog_object = catalog_extras
            .unwrap_or_default()
            .create_catalog_object(pages_id);
        if !layers.is_empty() {
            let oc_properties = OptionalContentProperties {
                ocgs: layers.into_values().map(|layer| layer.ocg_id).collect(),
                ..Default::default()
            };
            catalog_object.oc_properties = Some(oc_properties);
        }
        // Create Catalog object
        let catalog_id = document.add_object(catalog_object.into_dictionary());
        // Point the Root key to the Pages object
        document.trailer.root = Some(catalog_id);
        if let Some(info_dict) = info_dict {
            document.trailer.info = Some(info_dict);
        }
        Ok(document)
    }
}
