use ahash::HashMap;
use derive_more::derive::From;

use super::GlyphMetrics;

/// A trait for external font types. Being the types of loaders that this library supports.
///
/// This is used so we can support multiple font loading libraries.
pub trait ExternalLoadedFont {
    fn units_per_em(&self) -> u16;

    fn ascender(&self) -> i16;

    fn descender(&self) -> i16;

    fn italic_angle(&self) -> i64;

    fn glyph_id(&self, c: char) -> Option<u16>;
    /// Returns a map of glyph IDs to characters.
    ///
    /// Possibly change this to a non ddos resistent hash map
    /// I don't think it's necessary to have a ddos resistent hash map here
    fn glyph_ids(&self) -> HashMap<u16, char>;

    fn glyph_count(&self) -> u16;

    fn glyph_metrics(&self, glyph_id: u16) -> Option<GlyphMetrics>;

    fn font_bytes(&self) -> &[u8];
}

#[derive(Debug, Clone, PartialEq, From)]
pub enum ExternalFont {
    OwnedTtfParser(super::owned_ttf_parser::OwnedPdfTtfFont),
    StaticTtfParser(super::static_ttf_parser::StaticTtfFace),
}
impl ExternalLoadedFont for ExternalFont {
    fn units_per_em(&self) -> u16 {
        match self {
            ExternalFont::OwnedTtfParser(face) => face.units_per_em(),
            ExternalFont::StaticTtfParser(face) => face.units_per_em(),
        }
    }
    fn italic_angle(&self) -> i64 {
        match self {
            ExternalFont::OwnedTtfParser(face) => face.italic_angle(),
            ExternalFont::StaticTtfParser(face) => face.italic_angle(),
        }
    }

    fn ascender(&self) -> i16 {
        match self {
            ExternalFont::OwnedTtfParser(face) => face.ascender(),
            ExternalFont::StaticTtfParser(face) => face.ascender(),
        }
    }
    fn glyph_ids(&self) -> HashMap<u16, char> {
        match self {
            ExternalFont::OwnedTtfParser(face) => face.glyph_ids(),
            ExternalFont::StaticTtfParser(face) => face.glyph_ids(),
        }
    }

    fn descender(&self) -> i16 {
        match self {
            ExternalFont::OwnedTtfParser(face) => face.descender(),
            ExternalFont::StaticTtfParser(face) => face.descender(),
        }
    }

    fn glyph_id(&self, c: char) -> Option<u16> {
        match self {
            ExternalFont::OwnedTtfParser(face) => face.glyph_id(c),
            ExternalFont::StaticTtfParser(face) => face.glyph_id(c),
        }
    }

    fn glyph_count(&self) -> u16 {
        match self {
            ExternalFont::OwnedTtfParser(face) => face.glyph_count(),
            ExternalFont::StaticTtfParser(face) => face.glyph_count(),
        }
    }

    fn glyph_metrics(&self, glyph_id: u16) -> Option<GlyphMetrics> {
        match self {
            ExternalFont::OwnedTtfParser(face) => face.glyph_metrics(glyph_id),
            ExternalFont::StaticTtfParser(face) => face.glyph_metrics(glyph_id),
        }
    }

    fn font_bytes(&self) -> &[u8] {
        match self {
            ExternalFont::OwnedTtfParser(face) => face.font_bytes(),
            ExternalFont::StaticTtfParser(face) => face.font_bytes(),
        }
    }
}
