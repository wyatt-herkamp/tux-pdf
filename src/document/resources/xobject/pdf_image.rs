#[derive(Debug, Clone, PartialEq)]
pub struct PdfImage {
    pub pixels: ImageData,
    pub width: usize,
    pub height: usize,
    pub data_format: ImageFormat,
    pub tag: Vec<u8>,
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
