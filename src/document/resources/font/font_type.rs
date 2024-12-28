use ahash::{HashMap, HashMapExt};
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
pub(crate) trait TtfParserFont {
    fn as_face_ref(&self) -> &ttf_parser::Face<'_>;
}
impl<T> ExternalLoadedFont for T
where
    T: TtfParserFont,
{
    fn units_per_em(&self) -> u16 {
        self.as_face_ref().units_per_em()
    }

    fn ascender(&self) -> i16 {
        self.as_face_ref().ascender()
    }

    fn descender(&self) -> i16 {
        self.as_face_ref().descender()
    }

    fn glyph_id(&self, c: char) -> Option<u16> {
        self.as_face_ref().glyph_index(c).map(|x| x.0)
    }
    fn glyph_ids(&self) -> HashMap<u16, char> {
        let subtables = self
            .as_face_ref()
            .tables()
            .cmap
            .map(|cmap| cmap.subtables.into_iter().filter(|v| v.is_unicode()));
        let Some(subtables) = subtables else {
            return HashMap::new();
        };
        let mut map = HashMap::with_capacity(self.as_face_ref().number_of_glyphs().into());
        for subtable in subtables {
            subtable.codepoints(|c| {
                use std::convert::TryFrom as _;

                if let Ok(ch) = char::try_from(c) {
                    if let Some(idx) = subtable.glyph_index(c).filter(|idx| idx.0 > 0) {
                        map.entry(idx.0).or_insert(ch);
                    }
                }
            })
        }
        map
    }
    fn italic_angle(&self) -> i64 {
        let italic_angle = self.as_face_ref().italic_angle();
        // TODO: Figure out if this is the correct way
        italic_angle as i64
    }

    fn glyph_count(&self) -> u16 {
        self.as_face_ref().number_of_glyphs()
    }

    fn glyph_metrics(&self, glyph_id: u16) -> Option<GlyphMetrics> {
        let glyph_id = ttf_parser::GlyphId(glyph_id);
        if let Some(width) = self.as_face_ref().glyph_hor_advance(glyph_id) {
            let width = width as u32;
            let height = self
                .as_face_ref()
                .glyph_bounding_box(glyph_id)
                .map(|bbox| bbox.y_max - bbox.y_min - self.descender())
                .unwrap_or(1000) as u32;
            Some(GlyphMetrics { width, height })
        } else {
            None
        }
    }
    fn font_bytes(&self) -> &[u8] {
        self.as_face_ref().raw_face().data
    }
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
