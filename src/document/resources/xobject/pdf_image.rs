use image::{ColorType, DynamicImage, GenericImageView, ImageDecoder};
use lopdf::{dictionary, Object, Stream};
use utils::pull_alpha_out_of_rgb;
mod utils;
use crate::{
    graphics::{
        color::{ColorBits, ColorSpace},
        ctm::CurTransMat,
        size::Size,
    },
    units::Px,
    TuxPdfError,
};

#[derive(Debug, Clone, PartialEq)]
pub struct PdfImage {
    pub image: PdfXObjectImage,
    pub mask: Option<PdfXObjectImage>,
}

impl PdfImage {
    /// Load an image from an image decoder
    pub fn load_from_decoder<T>(image: T) -> Result<Self, TuxPdfError>
    where
        T: ImageDecoder,
    {
        let dim = image.dimensions();
        let color_type = image.color_type();
        let num_image_bytes = image.total_bytes();

        let mut image_data = vec![0; num_image_bytes as usize];
        image.read_image(&mut image_data)?;
        let (image, mask) = PdfXObjectImage::process_image(color_type, image_data, dim)?;
        Ok(PdfImage { image, mask })
    }
    /// Load an image from a dynamic image
    pub fn load_from_dynamic_image(image: DynamicImage) -> Result<Self, TuxPdfError> {
        let color_type = image.color();
        let (width, height) = image.dimensions();
        let image_data = image.into_bytes();
        let (image, mask) =
            PdfXObjectImage::process_image(color_type, image_data, (width, height))?;
        Ok(PdfImage { image, mask })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct PdfXObjectImage {
    /// Width of the image (original width, not scaled width)
    pub size: Size<Px>,
    /// Color space (Greyscale, RGB, CMYK)
    pub color_space: ColorSpace,
    pub bits_per_component: ColorBits,
    /// Should the image be interpolated when scaled?
    pub interpolate: bool,
    /// The actual data from the image
    pub image_data: Vec<u8>,
    /// Decompression filter for `image_data`, if `None` assumes uncompressed raw pixels in the expected color format.
    pub image_filter: Option<ImageFilter>,
    /// SoftMask for transparency, if `None` assumes no transparency. See page 444 of the adope pdf 1.4 reference
    pub smask: Option<lopdf::ObjectId>,
    /// Required bounds to clip the image, in unit space
    /// Default value: Identity matrix (`[1 0 0 1 0 0]`) - used when value is `None`
    pub clipping_bbox: Option<CurTransMat>,
}
impl PdfXObjectImage {
    /// Internal function to process image data
    #[doc(hidden)]
    pub fn process_image(
        color_type: ColorType,
        image_data: Vec<u8>,
        dim: (u32, u32),
    ) -> Result<(PdfXObjectImage, Option<PdfXObjectImage>), TuxPdfError> {
        let (color_type, image_data, smask_data) = match color_type {
            ColorType::Rgba8 => {
                let (rgb, alpha) = pull_alpha_out_of_rgb(image_data);
                (ColorType::Rgb8, rgb, Some(alpha))
            }
            _ => (color_type, image_data, None),
        };
        let color_bits = ColorBits::try_from(color_type)?;
        let color_space = ColorSpace::try_from(color_type)?;
        let size = Size {
            width: Px(dim.0 as i64),
            height: Px(dim.1 as i64),
        };
        let img = PdfXObjectImage {
            size,
            color_space,
            bits_per_component: color_bits,
            image_data,
            interpolate: true,
            image_filter: None,
            clipping_bbox: None,
            smask: None,
        };
        let img_mask = smask_data.map(|smask| PdfXObjectImage {
            size,
            color_space: ColorSpace::Greyscale,
            bits_per_component: ColorBits::Bit8,
            interpolate: false,
            image_data: smask,
            image_filter: None,
            clipping_bbox: None,
            smask: None,
        });
        Ok((img, img_mask))
    }
    /// Takes mut because I don't think I can drop all image details yet. This will hopefully be changed in the future.
    ///
    /// But for now it will take the image_data to hopefully prevent as much memory usage as possible.
    pub fn into_stream(&mut self) -> Result<Stream, TuxPdfError> {
        if self.image_data.is_empty() {
            todo!("Handle empty image data")
        }
        let color_space: Object = self.color_space.into();
        let bbox: Vec<Object> = self.clipping_bbox.unwrap_or(CurTransMat::Identity).into();
        let bbox: Object = Object::Array(bbox);

        let mut dictionary = dictionary! {
            "Type" => "XObject",
            "Subtype" => "Image",
            "Width" => self.size.width,
            "Height" => self.size.height,
            "ColorSpace" => color_space,
            "BitsPerComponent" => self.bits_per_component as i64,
            "Interpolate" => self.interpolate,
            "BBox" => bbox,
        };
        if let Some(smask) = self.smask {
            dictionary.set("SMask", Object::Reference(smask));
        }
        let data = std::mem::take(&mut self.image_data);

        Ok(Stream::new(dictionary, data))
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ImageData {
    // 8-bit image data
    U8(Vec<u8>),
    // 16-bit image data
    U16(Vec<u16>),
    // HDR image data
    F32(Vec<f32>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImageFormat {
    R8,
    RG8,
    RGB8,
    RGBA8,
    R16,
    RG16,
    RGB16,
    RGBA16,
    BGR8,
    BGRA8,
    RGBF32,
    RGBAF32,
}
/// Describes the format the image bytes are compressed with.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImageFilter {
    Ascii85,
    Lzw,
    DCT,
    JPX,
}
