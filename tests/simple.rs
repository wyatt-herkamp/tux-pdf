use test_utils::{destination_dir, fonts_dir, init_logger};
use tux_pdf::{
    document::{owned_ttf_parser::OwnedPdfTtfFont, PdfDocument},
    graphics::{text::TextStyle, Point, TextBlock},
    page::{page_sizes::A4, PdfPage},
    units::UnitType,
};
mod test_utils;

#[test]
fn simple_test() -> anyhow::Result<()> {
    init_logger();
    let mut doc = PdfDocument::new("My first document");
    test_utils::set_metadata_for_test(&mut doc);
    let roboto_font_reader =
        std::fs::File::open(fonts_dir().join("Roboto").join("Roboto-Regular.ttf"))?;
    let roboto_font = OwnedPdfTtfFont::new_from_reader(roboto_font_reader, 0)?;

    let roboto = doc.font_map().register_external_font(roboto_font)?;

    let font = doc
        .font_map()
        .register_builtin_font(tux_pdf::document::BuiltinFont::Helvetica);

    let mut page = PdfPage::new_from_page_size(A4);
    let test_0_0 = TextBlock {
        content: "Font is Built in Helvetica \n I am a new line!!!".into(),
        position: Point {
            x: 0.into(),
            y: 0.into(),
        },
        style: TextStyle {
            font_ref: font.clone(),
            ..Default::default()
        },
    };
    page.add_operation(test_0_0.into());
    let test_text = TextBlock {
        content: "Font is Built in Helvetica \n I am a new line!!!".into(),
        position: Point {
            x: 10f32.into(),
            y: 210f32.into(),
        },
        style: TextStyle {
            font_ref: font,
            ..Default::default()
        },
    };

    let roboto_text = TextBlock {
        content: "Font is Roboto".into(),
        position: Point {
            x: 20f32.into(),
            y: 180f32.into(),
        },
        style: TextStyle {
            font_ref: roboto,
            ..Default::default()
        },
    };
    page.add_operation(test_text.into());
    page.add_operation(roboto_text.into());
    doc.add_page(page);

    let mut pdf = doc.write_to_lopdf_document()?;
    let mut file = std::fs::File::create(destination_dir().join("simple.pdf"))?;
    pdf.save_to(&mut file)?;

    Ok(())
}
pub fn does_end_with_ttf(path: &std::path::Path) -> bool {
    path.extension().map_or(false, |ext| ext == "ttf")
}
#[test]
fn all_roboto() -> anyhow::Result<()> {
    let mut doc = PdfDocument::new("Roboto Examples");
    test_utils::set_metadata_for_test(&mut doc);

    let mut page = PdfPage::new_from_page_size(A4);
    let mut text_position = Point {
        x: 10f32.into(),
        y: A4.top_left_point().y - 10f32.pt(),
    };
    for fonts in std::fs::read_dir(fonts_dir().join("Roboto"))? {
        let fonts = fonts?;
        if !does_end_with_ttf(&fonts.path()) {
            continue;
        }
        let font_reader = std::fs::File::open(fonts.path())?;
        let read_font = OwnedPdfTtfFont::new_from_reader(font_reader, 0)?;
        let font = doc.font_map().register_external_font(read_font)?;
        let test_text = TextBlock {
            content: format!("Font is \n {}", fonts.file_name().to_string_lossy()).into(),
            position: text_position,
            style: TextStyle {
                font_ref: font,

                ..Default::default()
            },
        };
        page.add_operation(test_text.into());
        text_position.y -= 20f32;
    }

    doc.add_page(page);

    let mut pdf = doc.write_to_lopdf_document()?;
    let mut file = std::fs::File::create(destination_dir().join("all_roboto.pdf"))?;
    pdf.save_to(&mut file)?;

    Ok(())
}
