use std::fs::File;

use test_utils::{destination_dir, init_logger};
use tux_pdf_low::{
    content::{Content, Operation},
    document::PdfDocumentWriter,
    types::{Dictionary, Name, PdfString, PdfType, Stream},
};
mod test_utils;
#[test]
pub fn hello_world() -> anyhow::Result<()> {
    init_logger();
    let mut doc = PdfDocumentWriter::default();

    let pages_id = doc.next_object_id();
    let mut font_dictionary = Dictionary::new();
    font_dictionary.set("Type", Name::from("Font"));
    font_dictionary.set("Subtype", Name::from("Type1"));
    font_dictionary.set("BaseFont", Name::from("Courier"));
    let font_id = doc.add_object(font_dictionary);
    let mut font_resources = Dictionary::new();
    font_resources.set("F1", font_id);
    let mut resources = Dictionary::new();
    resources.set("Font", font_resources);
    let resources_id = doc.add_object(resources);
    let content = Content {
        operations: vec![
            Operation::new_empty("BT"),
            Operation::new("Tf", vec![Name::from("F1").into(), 48.into()]),
            Operation::new("Td", vec![100.into(), 600.into()]),
            Operation::new("Tj", vec![PdfString::literal("Hello World!").into()]),
            Operation::new_empty("ET"),
        ],
    };
    let content_stream = Stream::new(Dictionary::default(), content.write_to_vec()?);
    let content_id = doc.add_object(content_stream);

    // Page is a dictionary that represents one page of a PDF file.
    // Its required fields are "Type", "Parent" and "Contents".
    let mut first_page = Dictionary::new();
    first_page.set_type("Page");
    first_page.set("Parent", pages_id);
    first_page.set("Contents", content_id);
    first_page.set("MediaBox", vec![0, 0, 595, 842]);
    first_page.set("Resources", resources_id);
    let page_id = doc.add_object(first_page);

    let mut pages = Dictionary::new();
    pages.set_type("Pages");
    pages.set("Kids", vec![page_id]);
    pages.set("Count", 1);

    doc.set_object(pages_id, pages);

    let mut catalog = Dictionary::new();
    catalog.set_type("Catalog");
    catalog.set("Pages", pages_id);
    let catalog_id = doc.add_object(catalog);
    doc.trailer.root = Some(catalog_id);

    let mut target = File::create(destination_dir().join("hello_world.pdf"))?;

    doc.save(&mut target)?;
    Ok(())
}
