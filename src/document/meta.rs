use lopdf::{Dictionary, ObjectId};
use strum::{Display, EnumString};
use time::OffsetDateTime;

use crate::{
    time_impl::PdfDateTimeType,
    utils::{strum_into_name, IsEmpty},
};

use super::{conformance::PdfConformance, types::CatalogObject};

#[derive(Debug, PartialEq, Clone)]
pub struct PdfMetadata {
    /// Document information
    pub info: PdfDocumentInfo,
    /// XMP Metadata. Is ignored on save if the PDF conformance does not allow XMP
    pub xmp: Option<XmpMetadata>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PdfDocumentInfo {
    /// Is the document trapped?
    pub trapped: Option<bool>,
    /// PDF document version
    pub version: Option<i32>,
    /// Creation date of the document
    pub creation_date: Option<OffsetDateTime>,
    /// Modification date of the document
    pub modification_date: Option<OffsetDateTime>,
    /// Creation date of the metadata
    pub metadata_date: Option<OffsetDateTime>,
    /// PDF Standard
    pub conformance: PdfConformance,
    /// PDF document title
    pub document_title: String,
    /// PDF document author
    pub author: Option<String>,
    /// The creator of the document
    pub creator: Option<String>,
    /// The producer of the document
    pub producer: Option<String>,
    /// Keywords associated with the document
    pub keywords: Vec<String>,
    /// The subject of the document
    pub subject: Option<String>,
}
impl IsEmpty for PdfDocumentInfo {
    fn is_empty(&self) -> bool {
        self.document_title.is_empty()
            && self.author.is_empty()
            && self.creator.is_empty()
            && self.producer.is_empty()
            && self.keywords.is_empty()
            && self.subject.is_empty()
    }
}
impl From<PdfDocumentInfo> for Dictionary {
    fn from(value: PdfDocumentInfo) -> Self {
        let mut dict = Dictionary::new();
        dict.set("Title", value.document_title);
        if let Some(author) = value.author {
            dict.set("Author", author);
        }
        if let Some(creator) = value.creator {
            dict.set("Creator", creator);
        }
        if let Some(subject) = value.subject {
            dict.set("Subject", subject);
        }
        //TODO: Add keywords
        if let Some(producer) = value.producer {
            dict.set("Producer", producer);
        }
        if let Some(creation_date) = value.creation_date {
            dict.set("CreationDate", creation_date.format_into_object());
        }
        // TODO Modification date
        if let Some(trapped) = value.trapped {
            dict.set("Trapped", trapped);
        }
        dict
    }
}
impl Default for PdfDocumentInfo {
    fn default() -> Self {
        Self {
            trapped: None,
            version: None,
            conformance: PdfConformance::default(),
            document_title: String::new(),
            author: None,
            creator: None,
            producer: Some("tux-pdf".to_string()),
            keywords: Vec::new(),
            subject: None,
            creation_date: None,
            modification_date: None,
            metadata_date: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmpMetadata {
    /// Web-viewable or "default" or to be left empty. Usually "default".
    pub rendition_class: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, EnumString, Display, Default)]
pub enum PageLayout {
    /// Display one page at a time
    #[default]
    SinglePage,
    OneColumn,
    TwoColumnLeft,
    TwoColumnRight,
    TwoPageLeft,
    TwoPageRight,
}
strum_into_name!(PageLayout);

#[derive(Debug, Clone, PartialEq, Eq, EnumString, Display, Default)]
pub enum PageMode {
    /// Neither Document outline nor thumbnail images visible
    #[default]
    UseNone,
    UseOutlines,
    UseThumbs,
    FullScreen,
    UseOC,
    UseAttachments,
}
strum_into_name!(PageMode);
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CatalogInfo {
    pub page_layout: PageLayout,
    pub language: Option<String>,
    pub page_mode: PageMode,
}
impl CatalogInfo {
    pub fn create_catalog_object(self, pages: ObjectId) -> CatalogObject {
        let Self {
            page_layout,
            language,
            page_mode,
        } = self;
        CatalogObject {
            pages,
            page_layout,
            language,
            page_mode,
            ..Default::default()
        }
    }
}
