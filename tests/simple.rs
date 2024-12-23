use test_utils::{destination_dir, fonts_dir};
use tux_pdf::{
    document::PdfDocument,
    graphics::{
        text::{Text, TextStyle},
        Point,
    },
    page::{page_sizes::A4, PdfPage},
};
mod test_utils;

#[test]
fn simple_test() -> anyhow::Result<()> {
    let mut doc = PdfDocument::new("My first document");

    let roboto = doc.load_external_font(std::fs::File::open(
        fonts_dir().join("Roboto").join("Roboto-Regular.ttf"),
    )?)?;

    let font = doc.add_builtin_font(tux_pdf::document::BuiltinFont::Helvetica);

    let mut page = PdfPage::new_from_page_size(A4);
    let test_text = Text {
        value: "Font is Built in Helvetica".into(),
        position: Point {
            x: 10f32.into(),
            y: 210f32.into(),
        },
        style: TextStyle {
            font_ref: font,
            ..Default::default()
        },
    };

    let roboto_text = Text {
        value: "Font is Roboto".into(),
        position: Point {
            x: 10f32.into(),
            y: 200f32.into(),
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

    let mut page = PdfPage::new_from_page_size(A4);
    let mut text_position = Point {
        x: 10f32.into(),
        y: 210f32.into(),
    };
    for fonts in std::fs::read_dir(fonts_dir().join("Roboto"))? {
        let fonts = fonts?;
        if !does_end_with_ttf(&fonts.path()) {
            continue;
        }
        let font = doc.load_external_font(std::fs::File::open(fonts.path())?)?;
        let test_text = Text {
            value: format!("Font is {}", fonts.file_name().to_string_lossy()).into(),
            position: text_position,
            style: TextStyle {
                font_ref: font,
                ..Default::default()
            },
        };
        page.add_operation(test_text.into());
        text_position.y -= 10f32;
    }

    doc.add_page(page);

    let mut pdf = doc.write_to_lopdf_document()?;
    let mut file = std::fs::File::create(destination_dir().join("all_roboto.pdf"))?;
    pdf.save_to(&mut file)?;

    Ok(())
}
