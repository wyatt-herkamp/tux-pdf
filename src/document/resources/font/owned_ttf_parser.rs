//! A wrapper around [ttf_parser::Face] that is owned
//!
//! Based on [owned_ttf_parser](https://crates.io/crates/owned_ttf_parser)
//!
//! Why not just use owned_ttf_parser? Because I don't want to have to deal with another crate that is not a huge amount of code.
use ahash::{HashMap, HashMapExt};
use std::{fmt::Debug, marker::PhantomPinned, mem, ops::Deref, pin::Pin, sync::Arc};
use ttf_parser::Face;

use crate::TuxPdfError;

use super::{FontType, GlyphMetrics};
#[derive(Debug, Clone, PartialEq)]
pub struct OwnedPdfTtfFont {
    inner: Arc<OwnedFace>,
}

impl Deref for OwnedPdfTtfFont {
    type Target = OwnedFace;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl OwnedPdfTtfFont {
    /// Create a new instance of [PdfTtfFace] from a reader
    ///
    /// If index is not known use 0
    pub fn new_from_reader<R: std::io::Read>(mut font: R, index: u32) -> Result<Self, TuxPdfError> {
        let mut buf = Vec::<u8>::new();
        font.read_to_end(&mut buf)?;
        let font = OwnedFace::from_vec_inner(buf, index)?;

        Ok(Self::new(font))
    }
    pub fn new(face: OwnedFace) -> Self {
        Self {
            inner: Arc::new(face),
        }
    }
    pub fn as_face_ref(&self) -> &ttf_parser::Face<'_> {
        self.inner.as_face_ref()
    }
}
impl FontType for OwnedPdfTtfFont {
    fn units_per_em(&self) -> u16 {
        self.inner.as_face_ref().units_per_em()
    }

    fn ascender(&self) -> i16 {
        self.inner.as_face_ref().ascender()
    }

    fn descender(&self) -> i16 {
        self.inner.as_face_ref().descender()
    }

    fn glyph_id(&self, c: char) -> Option<u16> {
        self.inner.as_face_ref().glyph_index(c).map(|x| x.0)
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
        let italic_angle = self.inner.as_face_ref().italic_angle();
        // TODO: Figure out if this is the correct way
        italic_angle as i64
    }

    fn glyph_count(&self) -> u16 {
        self.inner.as_face_ref().number_of_glyphs()
    }

    fn glyph_metrics(&self, glyph_id: u16) -> Option<GlyphMetrics> {
        let glyph_id = ttf_parser::GlyphId(glyph_id);
        if let Some(width) = self.inner.as_face_ref().glyph_hor_advance(glyph_id) {
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
        self.inner.as_slice()
    }
}
pub trait AsFaceRef {
    /// Convert to a [`Face`](struct.Face.html) reference.
    fn as_face_ref(&self) -> &ttf_parser::Face<'_>;
}

impl AsFaceRef for ttf_parser::Face<'_> {
    #[inline]
    fn as_face_ref(&self) -> &ttf_parser::Face<'_> {
        self
    }
}

impl AsFaceRef for &ttf_parser::Face<'_> {
    #[inline]
    fn as_face_ref(&self) -> &ttf_parser::Face<'_> {
        self
    }
}
pub trait FaceMut {
    /// Sets a variation axis coordinate.
    ///
    /// See [`ttf_parser::Face::set_variation`].
    fn set_variation(&mut self, axis: ttf_parser::Tag, value: f32) -> Option<()>;
}
impl FaceMut for Face<'_> {
    #[inline]
    fn set_variation(&mut self, axis: ttf_parser::Tag, value: f32) -> Option<()> {
        ttf_parser::Face::set_variation(self, axis, value)
    }
}
impl FaceMut for &mut Face<'_> {
    #[inline]
    fn set_variation(&mut self, axis: ttf_parser::Tag, value: f32) -> Option<()> {
        ttf_parser::Face::set_variation(self, axis, value)
    }
}
pub struct OwnedFace(Pin<Box<SelfRefVecFace>>);
impl PartialEq for OwnedFace {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl Debug for OwnedFace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedFace").finish()
    }
}

impl OwnedFace {
    pub(crate) fn from_vec_inner(
        data: Vec<u8>,
        index: u32,
    ) -> Result<Self, ttf_parser::FaceParsingError> {
        let inner = SelfRefVecFace::try_from_vec(data, index)?;
        Ok(Self(inner))
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.0.data
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.0.into_vec()
    }
}
impl AsFaceRef for OwnedFace {
    #[inline]
    fn as_face_ref(&self) -> &ttf_parser::Face<'_> {
        self.0.inner_ref()
    }
}

impl AsFaceRef for &OwnedFace {
    #[inline]
    fn as_face_ref(&self) -> &ttf_parser::Face<'_> {
        self.0.inner_ref()
    }
}

struct SelfRefVecFace {
    data: Vec<u8>,
    face: Option<Face<'static>>,
    _pin: PhantomPinned,
}
impl SelfRefVecFace {
    fn try_from_vec(
        data: Vec<u8>,
        index: u32,
    ) -> Result<Pin<Box<Self>>, ttf_parser::FaceParsingError> {
        let face = Self {
            data,
            face: None,
            _pin: PhantomPinned,
        };
        let mut b = Box::pin(face);
        unsafe {
            let slice: &'static [u8] = std::slice::from_raw_parts(b.data.as_ptr(), b.data.len());
            let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut b);
            let mut_inner = mut_ref.get_unchecked_mut();
            mut_inner.face = Some(ttf_parser::Face::parse(slice, index)?);
        }
        Ok(b)
    }

    fn inner_ref<'a>(self: &'a Pin<Box<Self>>) -> &'a ttf_parser::Face<'a> {
        // Safety: if you have a ref `face` is always Some
        unsafe { self.face.as_ref().unwrap_unchecked() }
    }
    fn into_vec(self: Pin<Box<Self>>) -> Vec<u8> {
        // Safety: safe as `face` is dropped.
        let mut me = unsafe { Pin::into_inner_unchecked(self) };
        me.face.take(); // ensure dropped before taking `data`
        mem::take(&mut me.data)
    }
}
impl Drop for SelfRefVecFace {
    fn drop(&mut self) {
        self.face.take();
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        document::{font_tests::DebugFontType, owned_ttf_parser::OwnedFace},
        tests::does_end_with_ttf,
    };

    use super::OwnedPdfTtfFont;
    // TODO: Test for memory leaks
    #[test]
    fn parse_roboto() -> anyhow::Result<()> {
        let roboto_path = crate::tests::test_fonts_directory().join("Roboto");

        // Iterate over all files in the directory
        for entry in std::fs::read_dir(roboto_path)? {
            let entry = entry?;
            let path = entry.path();
            if !does_end_with_ttf(&path) {
                continue;
            }
            let data = std::fs::read(&path)?;
            let face = OwnedFace::from_vec_inner(data, 0)?;
            let pdf_ttf_font = OwnedPdfTtfFont::new(face);

            let debug = DebugFontType(&pdf_ttf_font);
            println!("--- Font ({}) ---", path.display());
            println!("{:#?}", debug);
        }
        Ok(())
    }
}
