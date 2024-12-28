use image::ColorType::{self, *};
use lopdf::Object;
use strum::Display;

use crate::{utils::strum_into_name, TuxPdfError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum ColorSpace {
    #[strum(serialize = "DeviceRGB")]
    Rgb,
    #[strum(serialize = "DeviceN")]
    Rgba,
    #[strum(serialize = "Indexed")]
    Palette,
    #[strum(serialize = "DeviceCMYK")]
    Cmyk,
    #[strum(serialize = "DeviceGray")]
    Greyscale,
    #[strum(serialize = "DeviceN")]
    GreyscaleAlpha,
}
strum_into_name!(ColorSpace);

impl TryFrom<ColorType> for ColorSpace {
    type Error = TuxPdfError;

    fn try_from(value: ColorType) -> Result<Self, Self::Error> {
        match value {
            L8 | L16 => Ok(ColorSpace::Greyscale),
            La8 | La16 => Ok(ColorSpace::GreyscaleAlpha),
            Rgb8 | Rgb16 => Ok(ColorSpace::Rgb),
            Rgba8 | Rgba16 => Ok(ColorSpace::Rgba),
            _ => Err(TuxPdfError::UnsupportedImageColorType(value)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum ColorBits {
    /// 8-bit color
    Bit8 = 8,
    /// 16-bit color
    Bit16 = 16,
}
impl From<ColorBits> for Object {
    fn from(value: ColorBits) -> Self {
        Object::Integer(value as i64)
    }
}
impl TryFrom<ColorType> for ColorBits {
    type Error = TuxPdfError;

    fn try_from(value: ColorType) -> Result<Self, Self::Error> {
        match value {
            L8 | La8 | Rgb8 | Rgba8 => Ok(ColorBits::Bit8),
            L16 | La16 | Rgb16 | Rgba16 => Ok(ColorBits::Bit16),
            _ => Err(TuxPdfError::UnsupportedImageColorType(value)),
        }
    }
}

impl From<ColorBits> for i64 {
    fn from(val: ColorBits) -> Self {
        match val {
            ColorBits::Bit8 => 8,
            ColorBits::Bit16 => 16,
        }
    }
}
