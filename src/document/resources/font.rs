use std::{
    borrow::Cow,
    collections::{BTreeMap, HashSet},
    fmt::Debug,
};
mod builtin;
mod font_type;
pub use builtin::*;
pub use font_type::*;
pub mod owned_ttf_parser;
pub mod static_ttf_parser;
use lopdf::{Dictionary, Object, Stream};
use tracing::debug;

use crate::{
    document::{
        types::{
            CIDSystemInfo, CidFontType2, FontDescriptorBuilder, FontEncoding, FontFlags,
            FontObject, PdfDirectoryType, Type0Font,
        },
        DocumentWriter,
    },
    graphics::size::Size,
    units::{Pt, UnitType},
    TuxPdfError,
};

use super::{IdType, ObjectMapType};

#[derive(Clone, Copy, Debug, Default)]
pub struct GlyphMetrics {
    /// The width of the glyph, typically the horizontal advance.
    pub width: u32,
    /// The height of the glyph, typically the difference between the ascent and the descent.
    pub height: u32,
}
impl GlyphMetrics {
    pub fn width_pt(&self, units_per_em: u16, font_size: Pt) -> Pt {
        let scale_factor = font_size.0 / units_per_em as f32;
        let width_in_points = self.width as f32 * scale_factor;
        Pt(width_in_points)
    }
    pub fn height_pt(&self, units_per_em: u16, font_size: Pt) -> Pt {
        let scale_factor = font_size.0 / units_per_em as f32;
        let height_in_points = self.height as f32 * scale_factor;
        Pt(height_in_points)
    }

    /// Converts the glyph size to PDF point size based on the font's units per em and the desired font size.
    pub fn glyph_size_in_points(&self, units_per_em: u16, font_size: Pt) -> (Pt, Pt) {
        let scale_factor = font_size.0 / units_per_em as f32;
        let width_in_points = self.width as f32 * scale_factor;
        let height_in_points = self.height as f32 * scale_factor;
        (Pt(width_in_points), Pt(height_in_points))
    }
}
/// A unique identifier for a font.
#[derive(Debug, PartialEq, Clone, Eq, PartialOrd, Ord)]
pub struct FontId(pub(crate) String);

impl IdType for FontId {
    fn new_random() -> Self {
        Self(crate::utils::random::random_character_string(32))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
    fn into_string(self) -> String {
        self.0
    }
    fn resource_category(&self) -> &'static str {
        "Font"
    }
}
impl TryFrom<String> for FontId {
    type Error = TuxPdfError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Must be 32 characters long and alphanumeric
        if value.len() != 32 {
            return Err(TuxPdfError::InvalidObjectId(value));
        }
        if !value.chars().all(|c| c.is_alphanumeric()) {
            return Err(TuxPdfError::InvalidObjectId(value));
        }
        Ok(Self(value))
    }
}
impl TryFrom<&str> for FontId {
    type Error = TuxPdfError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PdfFontMap {
    pub(crate) map: BTreeMap<FontId, ParsedFont>,
    pub(crate) registered_builtin_fonts: HashSet<BuiltinFont>,
}

impl ObjectMapType for PdfFontMap {
    type IdType = FontId;
    fn has_id(&self, id: &Self::IdType) -> bool {
        self.map.contains_key(id)
    }
}
impl PdfFontMap {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            registered_builtin_fonts: HashSet::new(),
        }
    }

    /// Register a built-in font.
    pub fn register_builtin_font(&mut self, font: BuiltinFont) -> FontRef {
        self.registered_builtin_fonts.insert(font);
        FontRef::Builtin(font)
    }
    pub fn is_built_in_registered(&self, font: &BuiltinFont) -> bool {
        self.registered_builtin_fonts.contains(font)
    }
    pub fn register_external_font(
        &mut self,
        font: impl Into<ExternalFont>,
    ) -> Result<FontRef, TuxPdfError> {
        let font = font.into();
        let font_id = self.new_id();
        self.map.insert(
            font_id.clone(),
            ParsedFont {
                font,
                font_name: font_id.0.clone(),
            },
        );

        Ok(FontRef::External(font_id))
    }
    /// Register an external font.
    pub fn register_parsed_external_font(&mut self, font: ParsedFont) -> FontRef {
        let font_id = self.new_id();
        self.map.insert(font_id.clone(), font);
        FontRef::External(font_id)
    }
    pub fn register_external_font_with_id(
        &mut self,
        font_id: FontId,
        font: ParsedFont,
    ) -> Result<(), TuxPdfError> {
        if self.has_id(&font_id) {
            return Err(TuxPdfError::ObjectCollectionError(font_id.0));
        }
        self.map.insert(font_id, font);

        Ok(())
    }

    pub fn get_external_font(&self, font_id: &FontId) -> Option<&ParsedFont> {
        self.map.get(font_id)
    }
    pub fn dictionary(&self, writer: &mut DocumentWriter) -> Dictionary {
        let mut dict = Dictionary::new();
        for (font_id, font) in &self.map {
            let font_dictionary = font.dictionary(writer);
            let font_direct_id = writer.insert_object(Object::from(font_dictionary));
            dict.set(font_id.0.clone(), font_direct_id);
        }
        for (font_id, font_def) in self
            .registered_builtin_fonts
            .iter()
            .map(|f| (f.dedicated_font_id(), Dictionary::from(*f)))
        {
            let font_direct_id = writer.insert_object(font_def.into());
            dict.set(font_id.to_owned(), font_direct_id);
        }
        dict
    }
    pub fn internal_font_type(&self, font_ref: &FontRef) -> Option<InternalFontTypes> {
        match font_ref {
            FontRef::External(id) => self.map.get(id).map(InternalFontTypes::External),
            FontRef::Builtin(builtin) => {
                if self.is_built_in_registered(builtin) {
                    Some(InternalFontTypes::Builtin(*builtin))
                } else {
                    None
                }
            }
        }
    }
}
/// Render Size Parameters. Used for calculating the size of text.
pub trait FontRenderSizeParams {
    /// The font size.
    fn font_size(&self) -> Pt;
    /// The character spacing.
    fn character_spacing(&self) -> Option<Pt>;
    /// The word spacing.
    fn word_spacing(&self) -> Option<Pt>;
    /// The text rise.
    fn text_rise(&self) -> Option<Pt>;
}
/// Font types
pub trait FontType {
    /// Calculate the size of the text.
    fn calculate_size_of_text<P: FontRenderSizeParams>(&self, text: &str, params: &P) -> Size;
    /// Calculate the height of the text.
    #[inline(always)]
    fn calculate_height_of_text<P: FontRenderSizeParams>(&self, text: &str, params: &P) -> Pt {
        self.calculate_size_of_text(text, params).height
    }
    /// Calculate the size of a character.
    fn size_of_char<P: FontRenderSizeParams>(&self, c: char, params: &P) -> Option<Size>;
    /// Encode the text.
    fn encode_text(&self, text: &str) -> Vec<u8>;
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InternalFontTypes<'font> {
    External(&'font ParsedFont),
    Builtin(BuiltinFont),
}
impl FontType for InternalFontTypes<'_> {
    fn encode_text(&self, text: &str) -> Vec<u8> {
        match self {
            InternalFontTypes::External(font) => font.encode_text(text),
            InternalFontTypes::Builtin(builtin) => builtin.encode_text(text),
        }
    }

    fn calculate_size_of_text<P: FontRenderSizeParams>(&self, text: &str, params: &P) -> Size {
        match self {
            InternalFontTypes::External(font) => font.calculate_size_of_text(text, params),
            InternalFontTypes::Builtin(builtin) => builtin.calculate_size_of_text(text, params),
        }
    }

    fn size_of_char<P: FontRenderSizeParams>(&self, c: char, params: &P) -> Option<Size> {
        match self {
            InternalFontTypes::External(font) => font.size_of_char(c, params),
            InternalFontTypes::Builtin(builtin) => builtin.size_of_char(c, params),
        }
    }
    fn calculate_height_of_text<P: FontRenderSizeParams>(&self, text: &str, params: &P) -> Pt {
        self.calculate_size_of_text(text, params).height
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFont {
    pub(crate) font: ExternalFont,
    pub(crate) font_name: String,
    // TODO
}
impl FontType for ParsedFont {
    fn calculate_size_of_text<P: FontRenderSizeParams>(&self, text: &str, params: &P) -> Size {
        let mut width = Pt::default();
        let mut height = f32::default();
        for c in text.chars() {
            if let Some(glyph_id) = self.get_glyph_id(c) {
                if let Some(metrics) = self.font.glyph_metrics(glyph_id) {
                    let (glyph_width, glyph_height) =
                        metrics.glyph_size_in_points(self.font.units_per_em(), params.font_size());
                    width += glyph_width;
                    height = height.max(glyph_height.0);
                }
            }
        }
        debug!(
            "Size of text {text:?} Width: {:#?}, Height: {:#?}",
            width, height
        );
        Size {
            width: width.pt(),
            height: height.pt(),
        }
    }
    fn size_of_char<P: FontRenderSizeParams>(&self, c: char, params: &P) -> Option<Size> {
        if let Some(glyph_id) = self.get_glyph_id(c) {
            if let Some(metrics) = self.font.glyph_metrics(glyph_id) {
                let (glyph_width, glyph_height) =
                    metrics.glyph_size_in_points(self.font.units_per_em(), params.font_size());
                return Some(Size {
                    width: glyph_width,
                    height: glyph_height,
                });
            }
        }
        None
    }

    fn encode_text(&self, text: &str) -> Vec<u8> {
        text.chars()
            .filter_map(|char| self.get_glyph_id(char))
            .flat_map(|glyph_id| vec![(glyph_id >> 8) as u8, (glyph_id & 255) as u8])
            .collect()
    }
}
impl ParsedFont {
    pub fn dictionary(&self, doc: &mut DocumentWriter) -> Dictionary {
        let bytes = self.font.font_bytes().to_vec();
        let font_stream = Stream::new(
            Dictionary::from_iter(vec![("Length1", Object::Integer(bytes.len() as i64))]),
            bytes,
        )
        .with_compression(false);

        let mut max_height = 0;
        // Total width of all characters
        let mut total_width = 0;
        // Widths (or heights, depends on self.vertical_writing)
        // of the individual characters, indexed by glyph id
        let mut widths = Vec::<(u32, u32)>::new();

        // Glyph IDs - (Unicode IDs - character width, character height)
        let mut cmap = BTreeMap::<u32, (u32, u32, u32)>::new();
        cmap.insert(0, (0, 1000, 1000));

        for (glyph_id, c) in self.font.glyph_ids() {
            if let Some(glyph_metrics) = self.font.glyph_metrics(glyph_id) {
                if glyph_metrics.height > max_height {
                    max_height = glyph_metrics.height;
                }

                total_width += glyph_metrics.width;
                cmap.insert(
                    glyph_id as u32,
                    (c as u32, glyph_metrics.width, glyph_metrics.height),
                );
            }
        }

        // Maps the character index to a unicode value - add this to the "ToUnicode" dictionary!
        //
        // To explain this structure: Glyph IDs have to be in segments where the first byte of the
        // first and last element have to be the same. A range from 0x1000 - 0x10FF is valid
        // but a range from 0x1000 - 0x12FF is not (0x10 != 0x12)
        // Plus, the maximum number of Glyph-IDs in one range is 100
        //
        // Since the glyph IDs are sequential, all we really have to do is to enumerate the vector
        // and create buckets of 100 / rest to 256 if needed

        let mut cur_first_bit: u16 = 0_u16; // current first bit of the glyph id (0x10 or 0x12) for example

        let mut all_cmap_blocks = Vec::new();

        {
            let mut current_cmap_block = Vec::new();

            for (glyph_id, unicode_width_tuple) in &cmap {
                if (*glyph_id >> 8) as u16 != cur_first_bit || current_cmap_block.len() >= 100 {
                    // end the current (beginbfchar endbfchar) block
                    all_cmap_blocks.push(current_cmap_block.clone());
                    current_cmap_block = Vec::new();
                    cur_first_bit = (*glyph_id >> 8) as u16;
                }

                let (unicode, width, _) = *unicode_width_tuple;
                current_cmap_block.push((*glyph_id, unicode));
                widths.push((*glyph_id, width));
            }

            all_cmap_blocks.push(current_cmap_block);
        }

        let cid_to_unicode_map =
            generate_cid_to_unicode_map(self.font_name.clone(), all_cmap_blocks);

        let cid_to_unicode_map_stream =
            Stream::new(Dictionary::new(), cid_to_unicode_map.as_bytes().to_vec());
        let cid_to_unicode_map_stream_id = doc.insert_object(cid_to_unicode_map_stream.into());

        // encode widths / heights so that they fit into what PDF expects
        // see page 439 in the PDF 1.7 reference
        // basically widths_list will contain objects like this:
        // 20 [21, 99, 34, 25]
        // which means that the character with the GID 20 has a width of 21 units
        // and the character with the GID 21 has a width of 99 units
        let mut widths_list = Vec::<Object>::new();
        let mut current_low_gid = 0;
        let mut current_high_gid = 0;
        let mut current_width_vec = Vec::<Object>::new();

        // scale the font width so that it sort-of fits into an 1000 unit square
        let percentage_font_scaling = 1000.0 / (self.font.units_per_em() as f32);

        for gid in 0..self.font.glyph_count() {
            if let Some(GlyphMetrics { width, .. }) = self.font.glyph_metrics(gid) {
                if gid == current_high_gid {
                    current_width_vec.push(Object::Integer(
                        (width as f32 * percentage_font_scaling) as i64,
                    ));
                    current_high_gid += 1;
                } else {
                    widths_list.push(Object::Integer(current_low_gid as i64));
                    widths_list.push(Object::Array(std::mem::take(&mut current_width_vec)));

                    current_width_vec.push(Object::Integer(
                        (width as f32 * percentage_font_scaling) as i64,
                    ));
                    current_low_gid = gid;
                    current_high_gid = gid + 1;
                }
            } else {
                continue;
            }
        }
        // push the last widths, because the loop is delayed by one iteration
        widths_list.push(Object::Integer(current_low_gid as i64));
        widths_list.push(Object::Array(std::mem::take(&mut current_width_vec)));
        let font_bbox = vec![
            0,
            (max_height as i64),
            (total_width as i64),
            (max_height as i64),
        ];
        let cid_system_info = CIDSystemInfo {
            registry: "Adobe".into(),
            ordering: "Identity".into(),
            supplement: 0,
        };
        let font_stream_id = doc.insert_object(font_stream.into());

        let font_descriptor = FontDescriptorBuilder::default()
            .font_name(&self.font_name)
            .ascent(self.font.ascender())
            .descent(self.font.descender())
            .cap_height(self.font.ascender())
            .italic_angle(self.font.italic_angle())
            .flags(FontFlags::ITALIC)
            .stem_v(80)
            .font_b_box(Some(font_bbox))
            .font_file2(font_stream_id)
            .build()
            .unwrap();
        let descriptor_dict: Dictionary = font_descriptor.into();
        let descriptor_id = doc.insert_object(descriptor_dict.into());
        let cid_font_two = FontObject {
            base_font: Cow::Borrowed(&self.font_name),
            encoding: None,
            sub_type: CidFontType2 {
                cid_system_info,
                font_descriptor: descriptor_id,
                dw: Some(1000),
                w: Some(widths_list),
                dw2: None,
                w2: None,
                cid_to_gid_map: None,
            },
        };
        let cid_font_two_dict = cid_font_two.into_dictionary();
        let font_primary = FontObject {
            base_font: Cow::Borrowed(&self.font_name),
            encoding: Some(FontEncoding::IdentityH),
            sub_type: Type0Font {
                descendant_fonts: vec![cid_font_two_dict],
                to_unicode: Some(cid_to_unicode_map_stream_id),
            },
        };

        font_primary.into_dictionary()
    }
    pub fn get_glyph_id(&self, c: char) -> Option<u16> {
        self.font.glyph_id(c)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum FontRef {
    External(FontId),
    Builtin(BuiltinFont),
}

impl FontRef {
    pub fn id(&self) -> &str {
        match self {
            FontRef::External(id) => &id.0,
            FontRef::Builtin(builtin) => builtin.dedicated_font_id(),
        }
    }
}
impl From<FontRef> for Object {
    fn from(font_ref: FontRef) -> Self {
        match font_ref {
            FontRef::External(id) => id.into(),
            FontRef::Builtin(builtin) => builtin.dedicated_font_id().into(),
        }
    }
}

fn generate_cid_to_unicode_map(face_name: String, all_cmap_blocks: Vec<Vec<(u32, u32)>>) -> String {
    let mut cid_to_unicode_map = gid_to_unicode_beg(face_name.as_str()).to_string();

    for cmap_block in all_cmap_blocks
        .into_iter()
        .filter(|block| !block.is_empty() || block.len() < 100)
    {
        cid_to_unicode_map.push_str(format!("{} beginbfchar\r\n", cmap_block.len()).as_str());
        for (glyph_id, unicode) in cmap_block {
            cid_to_unicode_map.push_str(format!("<{glyph_id:04x}> <{unicode:04x}>\n").as_str());
        }
        cid_to_unicode_map.push_str("endbfchar\r\n");
    }

    cid_to_unicode_map.push_str(GID_TO_UNICODE_END);
    cid_to_unicode_map
}

fn gid_to_unicode_beg(face_name: &str) -> String {
    format!(
        r#"/CIDInit /ProcSet findresource begin

12 dict beginInternalFontTypes

begincmap

%!PS-Adobe-3.0 Resource-CMap
%%DocumentNeededResources: procset CIDInit
%%IncludeResource: procset CIDInit

/CIDSystemInfo 3 dict dup begin
    /Registry (FontSpecific) def
    /Ordering ({0}) def
    /Supplement 0 def
end def

/CMapName /FontSpecific-{0} def
/CMapVersion 1 def
/CMapType 2 def
/WMode 0 def

1 begincodespacerange
<0000> <FFFF>
endcodespacerange
"#,
        face_name
    )
}

const GID_TO_UNICODE_END: &str = r#"endcmap
CMapName currentdict /CMap defineresource pop
end
end
"#;

#[cfg(test)]
pub(crate) mod font_tests {
    use std::fmt::Debug;

    use super::ExternalLoadedFont;

    pub struct DebugFontType<'font, T>(pub &'font T);
    impl<T> Debug for DebugFontType<'_, T>
    where
        T: ExternalLoadedFont,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("FontType")
                .field("font_internal_type", &std::any::type_name::<T>())
                .field("units_per_em", &self.0.units_per_em())
                .field("ascender", &self.0.ascender())
                .field("descender", &self.0.descender())
                .field("glyph_count", &self.0.glyph_count())
                .finish()
        }
    }
}
