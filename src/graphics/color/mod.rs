use std::borrow::Cow;

use derive_more::derive::From;
mod image;
pub use image::*;
use tracing::error;
use tux_pdf_low::types::Object;

use crate::document::IccProfileId;

use super::{operation_keys, PdfObjectType};
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

operation_keys!(ColorOperations => {
    /// Set Fill Color Device RGB
    ///
    /// ## Operands
    /// [r, g, b] where r, g, b are real numbers
    NonStrokingDeviceRgb => "rg",
    /// Set Stroke Color Device RGB
    ///
    /// ## Operands
    ///
    /// [r, g, b] where r, g, b are real numbers
    StrokeDeviceRgb => "RG",

    /// Set Stroking color space to DeviceGray
    ///
    /// ## Operands
    ///
    /// gray where gray is a real number
    /// 0 being black and 1 being white
    StrokeDeviceGray => "G",
    /// Set Non-stroking color space to DeviceGray
    ///
    /// ## Operands
    ///
    /// gray where gray is a real number
    /// 0 being black and 1 being white
    NonStrokingDeviceGray => "g",
    /// Set Stroking color space to DeviceCMYK
    ///
    /// ## Operands
    ///
    /// [c, m, y, k] where c, m, y, k are real numbers
    StrokingCMYK => "K",

    /// Set Non-stroking color space to DeviceCMYK
    ///
    /// ## Operands
    /// [c, m, y, k] where c, m, y, k are real numbers
    ///
    NonStrokingCMYK => "k"
});
#[derive(Debug, Clone, PartialEq, From)]
pub enum Color {
    Rgb(Rgb),
    Cmyk(Cmyk),
    Greyscale(Greyscale),
    SpotColor(SpotColor),
}
impl Color {
    pub fn has_color_profile(&self) -> bool {
        match self {
            Color::Rgb(rgb) => rgb.icc_profile.is_some(),
            Color::Cmyk(cmyk) => cmyk.icc_profile.is_some(),
            Color::Greyscale(greyscale) => greyscale.icc_profile.is_some(),
            Color::SpotColor(_) => false,
        }
    }
    pub fn get_color_profile(&self) -> Option<&IccProfileId> {
        match self {
            Color::Rgb(rgb) => rgb.icc_profile.as_ref(),
            Color::Cmyk(cmyk) => cmyk.icc_profile.as_ref(),
            Color::Greyscale(greyscale) => greyscale.icc_profile.as_ref(),
            Color::SpotColor(_) => None,
        }
    }
}
/// A trait to add color parameters to a shape
pub trait HasColorParams {
    /// Set the fill color of the shape
    ///
    /// Or the non-stroke color
    fn set_fill_color(&mut self, color: Color);
    /// Set the outline color of the shape
    ///
    /// Or the stroke color
    fn set_outline_color(&mut self, color: Color);

    fn with_fill_color(mut self, color: Color) -> Self
    where
        Self: Sized,
    {
        self.set_fill_color(color);
        self
    }
    fn with_outline_color(mut self, color: Color) -> Self
    where
        Self: Sized,
    {
        self.set_outline_color(color);
        self
    }

    fn take_fill_color(&mut self) -> Option<Color>;
    fn take_outline_color(&mut self) -> Option<Color>;

    fn write_colors(
        &mut self,
        resources: &crate::document::PdfResources,
        writer: &mut super::OperationWriter,
    ) -> Result<(), crate::TuxPdfError> {
        ColorWriter {
            outline_color: self.take_outline_color().map(Cow::Owned),
            fill_color: self.take_fill_color().map(Cow::Owned),
        }
        .write(resources, writer)
    }
}
/// A helper struct to write color operations to a pdf
pub struct ColorWriter<'style> {
    /// The color to use for the outline of the shape
    ///
    /// Or the stroke color
    pub outline_color: Option<Cow<'style, Color>>,
    /// The color to use for the fill of the shape
    ///
    /// Or the non-stroke color
    pub fill_color: Option<Cow<'style, Color>>,
}

impl PdfObjectType for ColorWriter<'_> {
    fn write(
        self,
        _: &crate::document::PdfResources,
        writer: &mut super::OperationWriter,
    ) -> Result<(), crate::TuxPdfError> {
        // TODO: Implement Color Space
        if let Some(outline_color) = self.outline_color {
            if let Some(icc_profile) = outline_color.get_color_profile() {
                error!("Color Profile not implemented yet: {:?}", icc_profile);
            }
            match outline_color.as_ref() {
                Color::Rgb(rgb) => {
                    writer.add_operation(ColorOperations::StrokeDeviceRgb, rgb.into());
                }
                Color::Greyscale(greyscale) => {
                    writer.add_operation(ColorOperations::StrokeDeviceGray, greyscale.into());
                }
                Color::Cmyk(cmyk) => {
                    writer.add_operation(ColorOperations::StrokingCMYK, cmyk.into());
                }
                _ => todo!("Implement SpotColor"),
            }
        }
        if let Some(fill_color) = self.fill_color {
            if let Some(icc_profile) = fill_color.get_color_profile() {
                error!("Color Profile not implemented yet: {:?}", icc_profile);
            }
            match fill_color.as_ref() {
                Color::Rgb(rgb) => {
                    writer.add_operation(ColorOperations::NonStrokingDeviceRgb, rgb.into());
                }
                Color::Greyscale(greyscale) => {
                    writer.add_operation(ColorOperations::NonStrokingDeviceGray, greyscale.into());
                }
                Color::Cmyk(cmyk) => {
                    writer.add_operation(ColorOperations::NonStrokingCMYK, cmyk.into());
                }
                _ => todo!("Implement SpotColor"),
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
impl From<&Cmyk> for Vec<Object> {
    fn from(cmyk: &Cmyk) -> Vec<Object> {
        vec![
            Object::Real(cmyk.c),
            Object::Real(cmyk.m),
            Object::Real(cmyk.y),
            Object::Real(cmyk.k),
        ]
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Greyscale {
    pub percent: f32,
    pub icc_profile: Option<IccProfileId>,
}
impl From<&Greyscale> for Vec<Object> {
    fn from(greyscale: &Greyscale) -> Vec<Object> {
        vec![Object::Real(greyscale.percent)]
    }
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
