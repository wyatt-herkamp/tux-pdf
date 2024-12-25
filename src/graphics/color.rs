use derive_more::derive::From;
use lopdf::Object;

use crate::document::IccProfileId;

use super::PdfOperationType;
macro_rules! default_rgb_colors {
    (
        $(
            $name:ident = $r:expr, $g:expr, $b:expr;
        )*
    ) => {
        $(
            pub const $name: Color = Color::Rgb(Rgb {
                r: $r,
                g: $g,
                b: $b,
                icc_profile: None,
            });
        )*
    };
}
default_rgb_colors! {
    RED_RGB = 1.0, 0.0, 0.0;
    GREEN_RGB = 0.0, 1.0, 0.0;
    BLUE_RGB = 0.0, 0.0, 1.0;
    BLACK_RGB = 0.0, 0.0, 0.0;
    WHITE_RGB = 1.0, 1.0, 1.0;
    GRAY_RGB = 0.5, 0.5, 0.5;
}
#[derive(Debug, Clone, PartialEq, From)]
pub enum Color {
    Rgb(Rgb),
    Cmyk(Cmyk),
    Greyscale(Greyscale),
    SpotColor(SpotColor),
}
/// A helper struct to write color operations to a pdf
pub struct ColorWriter<'style> {
    /// The color to use for the outline of the shape
    ///
    /// Or the stroke color
    pub outline_color: Option<&'style Color>,
    /// The color to use for the fill of the shape
    pub fill_color: Option<&'style Color>,
}

impl PdfOperationType for ColorWriter<'_> {
    fn write(
        self,
        _: &crate::document::PdfResources,
        writer: &mut super::OperationWriter,
    ) -> Result<(), crate::TuxPdfError> {
        if let Some(outline_color) = self.outline_color {
            match outline_color {
                Color::Rgb(rgb) => {
                    writer.add_operation(super::OperationKeys::ColorStrokeDeviceRgb, rgb.into());
                }
                _ => todo!("Implement other color types"),
            }
        }
        if let Some(fill_color) = self.fill_color {
            match fill_color {
                Color::Rgb(rgb) => {
                    writer.add_operation(super::OperationKeys::ColorFillDeviceRgb, rgb.into());
                }
                _ => todo!("Implement other color types"),
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub icc_profile: Option<IccProfileId>,
}
impl From<&Rgb> for Vec<Object> {
    fn from(rgb: &Rgb) -> Vec<Object> {
        vec![
            Object::Real(rgb.r),
            Object::Real(rgb.g),
            Object::Real(rgb.b),
        ]
    }
}
impl From<Rgb> for Vec<Object> {
    fn from(rgb: Rgb) -> Vec<Object> {
        vec![
            Object::Real(rgb.r),
            Object::Real(rgb.g),
            Object::Real(rgb.b),
        ]
    }
}
impl Rgb {
    pub const fn new_no_profile(r: f32, g: f32, b: f32) -> Self {
        Self {
            r,
            g,
            b,
            icc_profile: None,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Cmyk {
    pub c: f32,
    pub m: f32,
    pub y: f32,
    pub k: f32,
    pub icc_profile: Option<IccProfileId>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Greyscale {
    pub percent: f32,
    pub icc_profile: Option<IccProfileId>,
}

impl Greyscale {
    pub fn new(percent: f32, icc_profile: Option<IccProfileId>) -> Self {
        Self {
            percent,
            icc_profile,
        }
    }
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SpotColor {
    pub c: f32,
    pub m: f32,
    pub y: f32,
    pub k: f32,
}
