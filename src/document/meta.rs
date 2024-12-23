use time::OffsetDateTime;

use super::conformance::PdfConformance;

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
    pub trapped: bool,
    /// PDF document version
    pub version: u32,
    /// Creation date of the document
    pub creation_date: OffsetDateTime,
    /// Modification date of the document
    pub modification_date: OffsetDateTime,
    /// Creation date of the metadata
    pub metadata_date: OffsetDateTime,
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
    /// Identifier associated with the document
    pub identifier: String,
}
impl Default for PdfDocumentInfo {
    fn default() -> Self {
        Self {
            trapped: false,
            version: 1,
            conformance: PdfConformance::default(),
            document_title: String::new(),
            author: None,
            creator: None,
            producer: None,
            keywords: Vec::new(),
            subject: None,
            creation_date: OffsetDateTime::now_utc(),
            modification_date: OffsetDateTime::now_utc(),
            metadata_date: OffsetDateTime::now_utc(),
            identifier: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmpMetadata {
    /// Web-viewable or "default" or to be left empty. Usually "default".
    pub rendition_class: Option<String>,
}
