use bitflags::bitflags;
use derive_builder::Builder;
use std::borrow::Cow;
use tux_pdf_low::{
    dictionary,
    types::{Dictionary, Object, ObjectId},
};
/// Font descriptor
///
/// Section 9.8
#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct FontDescriptor<'font> {
    pub font_name: Cow<'font, str>,
    #[builder(default, setter(into, strip_option))]
    pub font_family: Option<Cow<'font, str>>,
    #[builder(default, setter(into, strip_option))]
    pub font_stretch: Option<Cow<'font, str>>,
    #[builder(default, setter(into, strip_option))]
    pub font_weight: Option<i64>,
    pub flags: FontFlags,

    pub font_b_box: Option<Vec<i64>>,

    pub italic_angle: i64,

    pub ascent: i64,
    pub descent: i64,
    #[builder(default, setter(into, strip_option))]
    pub leading: Option<i64>,
    #[builder(default, setter(into, strip_option))]
    pub cap_height: Option<i64>,
    #[builder(default, setter(into, strip_option))]
    pub x_height: Option<i64>,
    #[builder(default, setter(into, strip_option))]
    pub stem_v: Option<i64>,
    #[builder(default, setter(into, strip_option))]
    pub stem_h: Option<i64>,
    #[builder(default, setter(into, strip_option))]
    pub avg_width: Option<i64>,
    #[builder(default, setter(into, strip_option))]
    pub max_width: Option<i64>,
    #[builder(default, setter(into, strip_option))]
    pub missing_width: Option<i64>,
    #[builder(default, setter(into, strip_option))]
    pub font_file1: Option<ObjectId>,
    #[builder(default, setter(into, strip_option))]
    pub font_file2: Option<ObjectId>,
    #[builder(default, setter(into, strip_option))]
    pub font_file3: Option<ObjectId>,
    #[builder(default, setter(into, strip_option))]
    pub char_set: Option<Cow<'font, str>>,
}
impl From<FontDescriptor<'_>> for Dictionary {
    fn from(value: FontDescriptor<'_>) -> Self {
        let font_name = value.font_name.as_bytes().to_vec();
        let mut dict = dictionary!(
            "Type" => Object::name("FontDescriptor"),
            "FontName" => Object::name(font_name),
            "Flags" => value.flags.bits(),
            "ItalicAngle" => value.italic_angle,
            "Ascent" => value.ascent,
            "Descent" => value.descent
        );
        if let Some(font_family) = value.font_family {
            dict.set("FontFamily", Object::name(font_family.as_bytes().to_vec()));
        }
        if let Some(font_stretch) = value.font_stretch {
            dict.set(
                "FontStretch",
                Object::name(font_stretch.as_bytes().to_vec()),
            );
        }
        if let Some(font_weight) = value.font_weight {
            dict.set("FontWeight", font_weight);
        }
        if let Some(font_b_box) = value.font_b_box {
            let font_b_box: Vec<_> = font_b_box.into_iter().map(Object::Integer).collect();
            dict.set("FontBBox", font_b_box);
        }
        if let Some(leading) = value.leading {
            dict.set("Leading", leading);
        }
        if let Some(cap_height) = value.cap_height {
            dict.set("CapHeight", cap_height);
        }
        if let Some(x_height) = value.x_height {
            dict.set("XHeight", x_height);
        }
        if let Some(stem_v) = value.stem_v {
            dict.set("StemV", stem_v);
        }
        if let Some(stem_h) = value.stem_h {
            dict.set("StemH", stem_h);
        }
        if let Some(avg_width) = value.avg_width {
            dict.set("AvgWidth", avg_width);
        }
        if let Some(max_width) = value.max_width {
            dict.set("MaxWidth", max_width);
        }
        if let Some(missing_width) = value.missing_width {
            dict.set("MissingWidth", missing_width);
        }
        if let Some(font_file1) = value.font_file1 {
            dict.set("FontFile", Object::Reference(font_file1));
        }
        if let Some(font_file2) = value.font_file2 {
            dict.set("FontFile2", Object::Reference(font_file2));
        }
        if let Some(font_file3) = value.font_file3 {
            dict.set("FontFile3", Object::Reference(font_file3));
        }
        if let Some(char_set) = value.char_set {
            dict.set("CharSet", Object::name(char_set.as_bytes().to_vec()));
        }

        dict
    }
}

bitflags! {
    /// Flags for the font descriptor
    ///
    /// Section 9.8.2
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct FontFlags: i64 {
        /// Bit 1
        const FIXED_PITCH = 0b00000001;
        /// Bit 2
        const SERIF = 0b00000010;
        /// Bit 3
        const SYMBOLIC = 0b00000100;
        /// Bit 4
        const SCRIPT = 0b00001000;
        /// Bit 6
        const NON_SYMBOLIC = 0b00010000;
        /// Bit 7
        const ITALIC = 0b00100000;
        /// Bit 17
        const ALL_CAP = 0b00000001_00000000_00000000;
        /// Bit 18
        const SMALL_CAP =  0b00000010_00000000_00000000;
        /// Bit 19
        const FORCE_BOLD = 0b00000100_00000000_00000000;

    }
}

#[cfg(test)]
mod tests {
    use super::FontFlags;
    #[test]
    pub fn bit_flags() {
        let flags = FontFlags::from_bits_retain(32i64);

        flags.iter().for_each(|flag| {
            println!("{:?}", flag);
        });
    }
}
