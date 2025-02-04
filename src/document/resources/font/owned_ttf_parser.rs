//! A wrapper around [ttf_parser::Face] that is owned
//!
//! Based on [owned_ttf_parser](https://crates.io/crates/owned_ttf_parser)
//!
//! Why not just use owned_ttf_parser? Because I don't want to have to deal with another crate that is not a huge amount of code.
use std::{fmt::Debug, marker::PhantomPinned, mem, ops::Deref, pin::Pin, sync::Arc};
use ttf_parser::Face;

use crate::TuxPdfError;

use super::TtfParserFont;
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
impl TtfParserFont for OwnedPdfTtfFont {
    fn as_face_ref(&self) -> &ttf_parser::Face<'_> {
        self.inner.as_face_ref()
    }
}

impl OwnedPdfTtfFont {
    /// Create a new instance of [OwnedPdfTtfFont] from a reader
    ///
    /// If index is not known use 0
    pub fn new_from_reader<R: std::io::Read>(mut font: R, index: u32) -> Result<Self, TuxPdfError> {
        let mut buf = Vec::<u8>::new();
        font.read_to_end(&mut buf)?;
        let font = OwnedFace::from_vec_inner(buf, index)?;

        Ok(Self::new(font))
    }
    /// Create a new instance of [OwnedPdfTtfFont] from a vector of bytes
    pub fn new_vec(data: Vec<u8>, index: u32) -> Result<Self, TuxPdfError> {
        let font = OwnedFace::from_vec_inner(data, index)?;
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
    pub fn as_face_ref(&self) -> &ttf_parser::Face<'_> {
        self.0.inner_ref()
    }
    pub fn into_vec(self) -> Vec<u8> {
        self.0.into_vec()
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

    use crate::document::{font_tests::DebugFontType, owned_ttf_parser::OwnedFace};

    use super::OwnedPdfTtfFont;
    // TODO: Test for memory leaks
    #[test]
    fn parse_roboto() -> anyhow::Result<()> {
        let roboto_path = crate::tests::fonts_dir().join("Roboto");

        // Iterate over all files in the directory
        for entry in std::fs::read_dir(roboto_path)? {
            let entry = entry?;
            let path = entry.path();
            if !crate::tests::does_end_with_ttf(&path) {
                continue;
            }
            let data = std::fs::read(&path)?;
            let face = OwnedFace::from_vec_inner(data, 0)?;
            let pdf_ttf_font = OwnedPdfTtfFont::new(face);

            let debug = DebugFontType(&pdf_ttf_font);
            println!("--- Font ({}) ---", path.display());
            for ele in pdf_ttf_font.as_face_ref().names().into_iter() {
                println!("{:?}", ele);
            }
            println!("{:#?}", debug);
        }
        Ok(())
    }
}
