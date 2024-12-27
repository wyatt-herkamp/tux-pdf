//! Module regulating the comparison and feature sets / allowed plugins of a PDF document
//!
//!
//! [PDF/X Versions](https://en.wikipedia.org/wiki/PDF/X)
//!
//! [PDF/A Versions](https://en.wikipedia.org/wiki/PDF/A)

use std::borrow::Cow;
pub static A1B_2005_PDF_1_4_FEATURES: &[PdfConformanceFeatures] = &[];

#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(non_camel_case_types)]
pub enum PdfConformance {
    A1B_2005_PDF_1_4,
    Custom(CustomPdfConformance),
}

// default: save on file size
impl Default for PdfConformance {
    fn default() -> Self {
        Self::Custom(CustomPdfConformance::default())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CustomPdfConformance {
    /// Identifier for this conformance
    pub identifier: Option<Cow<'static, str>>,

    pub features: Vec<PdfConformanceFeatures>,
}

impl Default for CustomPdfConformance {
    fn default() -> Self {
        CustomPdfConformance {
            identifier: None,
            features: vec![
                PdfConformanceFeatures::ContentJpeg,
                PdfConformanceFeatures::AllowsPDFLayers,
            ],
        }
    }
}

impl PdfConformance {
    /// Get the identifier string for PDF
    pub fn get_identifier_string(&self) -> &str {
        match self {
            PdfConformance::A1B_2005_PDF_1_4 => "PDF/A-1b:2005",
            PdfConformance::Custom(c) => c.identifier.as_deref().unwrap_or(""),
        }
    }
    /// Check if a feature is allowed in this conformance
    pub fn has_feature(&self, feature: PdfConformanceFeatures) -> bool {
        match self {
            PdfConformance::Custom(c) => c.features.contains(&feature),
            PdfConformance::A1B_2005_PDF_1_4 => A1B_2005_PDF_1_4_FEATURES.contains(&feature),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PdfConformanceFeatures {
    Content3D,
    ContentVideo,
    ContentAudio,
    ContentJavascript,
    ContentJpeg,
    RequiresXMPMetadata,
    AllowsDefaultFonts,
    RequiresICCProfile,
    AllowsPDFLayers,
}
