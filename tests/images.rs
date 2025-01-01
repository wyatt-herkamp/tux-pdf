use std::{fs::File, io::BufReader};

use image::codecs::png::PngDecoder;
use test_utils::{destination_dir, fonts_dir, images_dir, init_logger};
use tux_pdf::{
    document::{owned_ttf_parser::OwnedPdfTtfFont, PdfDocument, PdfXObjectImage},
    graphics::{image::PdfImage, text::TextStyle, LayerType, PdfPosition, TextBlock},
    page::{page_sizes::A4, PdfPage},
    units::UnitType,
};
mod test_utils;
#[test]
pub fn basic_image() -> anyhow::Result<()> {
    init_logger();
    let mut doc = PdfDocument::new("Image Test");
    test_utils::set_metadata_for_test(&mut doc);
    let code_image_reader = BufReader::new(File::open(images_dir().join("code_image.png"))?);
    let roboto_font_reader =
        std::fs::File::open(fonts_dir().join("Roboto").join("Roboto-Regular.ttf"))?;
    let roboto_font = OwnedPdfTtfFont::new_from_reader(roboto_font_reader, 0)?;
    let roboto_font_ref = doc.resources.fonts.register_external_font(roboto_font)?;
    let image = PngDecoder::new(code_image_reader)?;

    let pdf_image = PdfXObjectImage::load_from_decoder(image)?;
    let code_image_ref = doc.add_xobject(pdf_image);

    let mut page = PdfPage::new_from_page_size(A4);

    let text = TextBlock::from("My code image")
        .with_style(TextStyle {
            font_ref: roboto_font_ref,
            font_size: 12.0.pt(),
            ..Default::default()
        })
        .with_position(PdfPosition::new(10.0.pt(), 800.0.pt()));
    page.add_to_layer(text.into())?;

    let image = PdfImage::new(code_image_ref)
        .with_position(PdfPosition::new(10.0.pt(), 750.0.pt()))
        .with_scape(2f32, 2f32)
        .with_dpi(300.0);

    page.add_to_layer(image.into())?;

    doc.add_page(page);

    let mut pdf = doc.save_to_lopdf_document()?;

    let mut file = File::create(destination_dir().join("basic_image.pdf"))?;

    pdf.save(&mut file)?;
    Ok(())
}
