use rand::Rng;
use tux_pdf::{
    document::PdfDocument,
    graphics::color::{Color, Rgb},
};
include!("test_utils_external.rs");

#[allow(dead_code)]
pub fn random_color() -> Color {
    let mut rng = rand::thread_rng();
    Color::Rgb(Rgb {
        r: rng.gen_range(0.0..1.0),
        g: rng.gen_range(0.0..1.0),
        b: rng.gen_range(0.0..1.0),
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
