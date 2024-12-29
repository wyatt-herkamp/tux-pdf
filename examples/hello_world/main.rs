use std::{fs::File, io::Cursor, path::PathBuf};

use clap::Parser;
use image::codecs::png::PngDecoder;
use tux_pdf::{
    document::{static_ttf_parser::StaticTtfFace, PdfDocument, PdfXObjectImage},
    graphics::{image::PdfImage, text::TextStyle, LayerType, PdfPosition, TextBlock},
    page::{page_sizes::A4, PdfPage},
    units::UnitType,
};
static ROBOTO_FONT: &[u8] = include_bytes!("../../tests/fonts/Roboto/Roboto-Regular.ttf");
static IMAGE: &[u8] = include_bytes!("code.png");

#[derive(Debug, Clone, Parser)]
struct HelloWorld {
    #[clap(default_value = "hello_world.pdf")]
    output_file: PathBuf,
}
pub fn main() -> anyhow::Result<()> {
    let args = HelloWorld::parse();

    let mut doc = PdfDocument::new("Hello World");
    let code_image_reader = Cursor::new(IMAGE);

    let roboto_font = StaticTtfFace::from_slice(ROBOTO_FONT, 0)?;
    let roboto_font_ref = doc.resources.fonts.register_external_font(roboto_font)?;
    let image = PngDecoder::new(code_image_reader)?;

    let pdf_image = PdfXObjectImage::load_from_decoder(image)?;
    let code_image_ref = doc.add_xobject(pdf_image);

    let mut page = PdfPage::new_from_page_size(A4);

    let text = TextBlock::from("Hello World")
        .with_style(TextStyle {
            font_ref: roboto_font_ref,
            font_size: 12.0.pt(),
            ..Default::default()
        })
        .with_position(PdfPosition::new(10.0.pt(), 800.0.pt()));
    page.add_to_layer(text.into())?;

    let image = PdfImage::new(code_image_ref)
        .with_position(PdfPosition::new(10.0.pt(), 100.0.pt()))
        .with_scape(2f32, 2f32)
        .with_dpi(300.0);

    page.add_to_layer(image.into())?;

    doc.add_page(page);

    let mut pdf = doc.save_to_lopdf_document()?;

    let mut file = File::create(args.output_file)?;

    pdf.save_to(&mut file)?;
    Ok(())
}
