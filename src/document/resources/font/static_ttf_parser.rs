/*!
 * Uses the `ttf-parser` crate to parse a TTF file and extract the font information.
 * This one unlike owned_ttf_parser uses a static lifetime for the font data.
 *
 * Good when you are using embedded fonts in the binary.
*/

use ttf_parser::Face;

use crate::TuxPdfError;

use super::TtfParserFont;
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
    pub fn from_slice(data: &'static [u8], index: u32) -> Result<Self, TuxPdfError> {
        let face = Face::parse(data, index)?;
        Ok(Self {
            inner: Box::new(face),
        })
    }

    pub fn as_face_ref(&self) -> &ttf_parser::Face<'static> {
        &self.inner
    }
}
impl TtfParserFont for StaticTtfFace {
    fn as_face_ref(&self) -> &ttf_parser::Face<'_> {
        self.as_face_ref()
    }
}
#[cfg(test)]
mod tests {

    use crate::document::{font_tests::DebugFontType, static_ttf_parser::StaticTtfFace};

    // TODO: Test for memory leaks
    #[test]
    fn parse_roboto_regular() -> anyhow::Result<()> {
        static TTF_DATA: &[u8] =
            include_bytes!("../../../../tests/fonts/Roboto/Roboto-Regular.ttf");

        let face = StaticTtfFace::from_slice(TTF_DATA, 0)?;

        let debug = DebugFontType(&face);
        println!("--- Font (tests/fonts/Roboto/Roboto-Regular.ttf) ---");
        println!("{:#?}", debug);

        Ok(())
    }
}
