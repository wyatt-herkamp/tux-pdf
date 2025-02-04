use rand::{rngs::StdRng, Rng, SeedableRng};
use tux_pdf::{
    document::PdfDocument,
    graphics::color::{Color, Rgb},
};
include!("test_utils_external.rs");

#[allow(dead_code)]
pub fn random_color() -> Color {
    let mut random = StdRng::from_os_rng();

    Color::Rgb(Rgb {
        r: random.random_range(0.0..1.0),
        g: random.random_range(0.0..1.0),
        b: random.random_range(0.0..1.0),
        icc_profile: None,
    })
}
#[allow(dead_code)]
pub fn set_metadata_for_test(document: &mut PdfDocument) {
    document.metadata.info.author = Some("tux-pdf tests".to_string());
    document.metadata.info.creator = Some("tux-pdf tests".to_string());
    document.metadata.info.creation_date =
        Some(time::OffsetDateTime::now_local().unwrap_or(time::OffsetDateTime::now_utc()));
}
#[allow(dead_code)]
pub fn create_test_document(name: &str) -> PdfDocument {
    let mut doc = PdfDocument::new(name);
    set_metadata_for_test(&mut doc);
    doc
}
#[allow(dead_code)]
pub fn save_pdf_doc(doc: PdfDocument, test_name: &str) -> anyhow::Result<()> {
    let save_location = destination_dir().join(format!("{}.pdf", test_name));
    let mut file = std::fs::File::create(save_location)?;
    let pdf = doc.write_into_pdf_document_writer()?;

    pdf.save(&mut file)?;

    Ok(())
}
