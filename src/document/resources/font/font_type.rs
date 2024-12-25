use ahash::HashMap;
use derive_more::derive::From;

use super::GlyphMetrics;

/// A trait for external font types.
///
/// This is used so we can support multiple font loading libraries.
pub trait FontType {
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
pub enum DynFontTypes {
    OwnedTtfParser(super::owned_ttf_parser::OwnedPdfTtfFont),
    StaticTtfParser(super::static_ttf_parser::StaticTtfFace),
}
impl FontType for DynFontTypes {
    fn units_per_em(&self) -> u16 {
        match self {
            DynFontTypes::OwnedTtfParser(face) => face.units_per_em(),
            DynFontTypes::StaticTtfParser(face) => face.units_per_em(),
        }
    }
    fn italic_angle(&self) -> i64 {
        match self {
            DynFontTypes::OwnedTtfParser(face) => face.italic_angle(),
            DynFontTypes::StaticTtfParser(face) => face.italic_angle(),
        }
    }

    fn ascender(&self) -> i16 {
        match self {
            DynFontTypes::OwnedTtfParser(face) => face.ascender(),
            DynFontTypes::StaticTtfParser(face) => face.ascender(),
        }
    }
    fn glyph_ids(&self) -> HashMap<u16, char> {
        match self {
            DynFontTypes::OwnedTtfParser(face) => face.glyph_ids(),
            DynFontTypes::StaticTtfParser(face) => face.glyph_ids(),
        }
    }

    fn descender(&self) -> i16 {
        match self {
            DynFontTypes::OwnedTtfParser(face) => face.descender(),
            DynFontTypes::StaticTtfParser(face) => face.descender(),
        }
    }

    fn glyph_id(&self, c: char) -> Option<u16> {
        match self {
            DynFontTypes::OwnedTtfParser(face) => face.glyph_id(c),
            DynFontTypes::StaticTtfParser(face) => face.glyph_id(c),
        }
    }

    fn glyph_count(&self) -> u16 {
        match self {
            DynFontTypes::OwnedTtfParser(face) => face.glyph_count(),
            DynFontTypes::StaticTtfParser(face) => face.glyph_count(),
        }
    }

    fn glyph_metrics(&self, glyph_id: u16) -> Option<GlyphMetrics> {
        match self {
            DynFontTypes::OwnedTtfParser(face) => face.glyph_metrics(glyph_id),
            DynFontTypes::StaticTtfParser(face) => face.glyph_metrics(glyph_id),
        }
    }

    fn font_bytes(&self) -> &[u8] {
        match self {
            DynFontTypes::OwnedTtfParser(face) => face.font_bytes(),
            DynFontTypes::StaticTtfParser(face) => face.font_bytes(),
        }
    }
}
