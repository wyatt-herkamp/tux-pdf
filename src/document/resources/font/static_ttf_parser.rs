/*!
 * Uses the `ttf-parser` crate to parse a TTF file and extract the font information.
 * This one unlike owned_ttf_parser uses a static lifetime for the font data.
 *
 * Good when you are using embedded fonts in the binary.
*/

use ahash::{HashMap, HashMapExt};

use ttf_parser::Face;

use super::{FontType, GlyphMetrics};
#[derive(Debug, Clone)]
pub struct StaticTtfFace {
    inner: Box<Face<'static>>,
}
impl PartialEq for StaticTtfFace {
    fn eq(&self, other: &Self) -> bool {
        self.inner.raw_face().data == other.inner.raw_face().data
    }
}

impl StaticTtfFace {
    pub fn from_slice(
        data: &'static [u8],
        index: u32,
    ) -> Result<Self, ttf_parser::FaceParsingError> {
        let face = Face::parse(data, index)?;
        Ok(Self {
            inner: Box::new(face),
        })
    }

    pub fn as_face_ref(&self) -> &ttf_parser::Face<'static> {
        &self.inner
    }
}
impl FontType for StaticTtfFace {
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
        self.inner.raw_face().data
    }
}
