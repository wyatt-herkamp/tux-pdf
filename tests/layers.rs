use test_utils::{create_test_document, fonts_dir, init_logger, save_pdf_doc};
use tux_pdf::{
    document::owned_ttf_parser::OwnedPdfTtfFont,
    graphics::{text::TextStyle, LayerType, PdfPosition, TextBlock},
    page::{page_sizes::A4, PdfPage},
    units::UnitType,
};
mod test_utils;
#[test]
pub fn one_page_two_layers() -> anyhow::Result<()> {
    let mut doc = create_test_document("one_page_two_layers");

    let roboto_font_reader =
        std::fs::File::open(fonts_dir().join("Roboto").join("Roboto-Regular.ttf"))?;

    let roboto_font = OwnedPdfTtfFont::new_from_reader(roboto_font_reader, 0)?;

    let roboto_font_ref = doc.resources.fonts.register_external_font(roboto_font)?;

    let font = doc
        .resources
        .fonts
        .register_builtin_font(tux_pdf::document::BuiltinFont::Helvetica);

    let mut page = PdfPage::new_from_page_size(A4);

    {
        let layer_one_text = TextBlock {
            content: "I am on layer 1".into(),
            position: PdfPosition {
                x: 250f32.into(),
                y: 250f32.into(),
            },
            style: TextStyle {
                font_ref: font.clone(),
                ..Default::default()
            },
        };
        let layer_one_ref = doc.create_layer("Layer 1");
        let layer = doc.resources.layers.get_layer_mut(&layer_one_ref).unwrap();
        layer.add_to_layer(layer_one_text.into())?;
        page.add_layer(layer_one_ref);
    }

    {
        let layer_two_text = TextBlock {
            content: "I am on layer 2".into(),
            position: PdfPosition {
                x: 250f32.into(),
                y: 300f32.into(),
            },
            style: TextStyle {
                font_ref: roboto_font_ref,
                ..Default::default()
            },
        };

        let layer_two_ref = doc.create_layer("Layer 2");
        let layer = doc.resources.layers.get_layer_mut(&layer_two_ref).unwrap();
        layer.add_to_layer(layer_two_text.into())?;
        page.add_layer(layer_two_ref);
    }

    doc.add_page(page);

    save_pdf_doc(doc, "one_page_two_layers")?;
    Ok(())
}
