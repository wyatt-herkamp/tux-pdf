use std::{fs::File, io::BufReader};

use image::codecs::png::PngDecoder;
use test_utils::{destination_dir, does_end_with_ttf, fonts_dir, images_dir, init_logger};
use tux_pdf::{
    document::{
        ExternalLoadedFont, PdfDocument, PdfXObjectImage, owned_ttf_parser::OwnedPdfTtfFont,
    },
    graphics::{
        LayerType, PdfPosition, TextBlock, TextBlockContent, TextItem, TextLine, image::PdfImage,
        text::TextStyle,
    },
    layouts::LayoutItemType,
    page::{PdfPage, page_sizes::A4},
    units::UnitType,
};
mod test_utils;
/// Shows some simple text with both a built in font and an external font
#[test]
fn simple_test() -> anyhow::Result<()> {
    init_logger();
    let mut doc = PdfDocument::new("My first document");
    //doc.metadata.set_open_action(JavascriptAction::from(
    //   r#"app.alert("PDF Allows Javascript")"#,
    //));
    test_utils::set_metadata_for_test(&mut doc);
    let noto_color_emoji = std::fs::File::open(
        fonts_dir()
            .join("Noto_Color_Emoji")
            .join("NotoColorEmoji-Regular.ttf"),
    )?;
    let noto_color_emoji = OwnedPdfTtfFont::new_from_reader(noto_color_emoji, 0)?;

    let noto_color_emoji = doc
        .font_map()
        .register_external_font_with_name("EmojiFont", noto_color_emoji)?;

    let roboto_font_reader =
        std::fs::File::open(fonts_dir().join("Roboto").join("Roboto-Regular.ttf"))?;
    let roboto_font = OwnedPdfTtfFont::new_from_reader(roboto_font_reader, 0)?;

    let roboto = doc.font_map().register_external_font(roboto_font)?;

    let font = doc
        .font_map()
        .register_builtin_font(tux_pdf::document::BuiltinFont::Helvetica);

    let mut page = PdfPage::new_from_page_size(A4);
    let helvetica_text_block = {
        TextBlockContent::default()
            .add_line(
                TextLine::from("Font is Built in Helvetica ")
                    .add_item(TextItem::new("😊").with_font(noto_color_emoji)),
            )
            .add_line(TextLine::from("I am a new line!!!"))
    };
    let helvetica_text = TextBlock::from(helvetica_text_block).with_style(TextStyle {
        font_ref: font.clone(),
        font_size: 16f32.pt(),
        ..Default::default()
    });
    let roboto_text =
        TextBlock::from("This font is Roboto  \n I am a new line!!!").with_style(TextStyle {
            font_ref: roboto.clone(),
            font_size: 16f32.pt(),

            ..Default::default()
        });
    page.add_to_layer(
        roboto_text
            .clone()
            .with_position((200f32.pt(), 0f32.pt()).into()),
    )?;
    page.add_to_layer(
        helvetica_text
            .clone()
            .with_position((0f32.pt(), 0f32.pt()).into()),
    )?;

    page.add_to_layer(helvetica_text.with_position((10f32.pt(), 800f32.pt()).into()))?;
    page.add_to_layer(roboto_text.with_position((200.pt(), 800f32.pt()).into()))?;
    doc.add_page(page);

    let pdf = doc.write_into_pdf_document_writer()?;
    let mut file = std::fs::File::create(destination_dir().join("simple.pdf"))?;
    pdf.save(&mut file)?;

    Ok(())
}
/// Tests emoji rendering with RasterizeToImage mode
#[test]
fn emoji_rasterize_test() -> anyhow::Result<()> {
    init_logger();
    let mut doc = PdfDocument::new("Emoji Rasterize Test");
    test_utils::set_metadata_for_test(&mut doc);

    // Enable rasterized emoji rendering
    doc.set_emoji_render_mode(tux_pdf::document::EmojiRenderMode::rasterize());

    let noto_color_emoji = std::fs::File::open(
        fonts_dir()
            .join("Noto_Color_Emoji")
            .join("NotoColorEmoji-Regular.ttf"),
    )?;
    let noto_color_emoji = OwnedPdfTtfFont::new_from_reader(noto_color_emoji, 0)?;
    let noto_color_emoji = doc
        .font_map()
        .register_external_font_with_name("EmojiFont", noto_color_emoji)?;

    let roboto_font_reader =
        std::fs::File::open(fonts_dir().join("Roboto").join("Roboto-Regular.ttf"))?;
    let roboto_font = OwnedPdfTtfFont::new_from_reader(roboto_font_reader, 0)?;
    let roboto = doc.font_map().register_external_font(roboto_font)?;

    let mut page = PdfPage::new_from_page_size(A4);

    // Mixed text with emoji using rasterized mode
    let mixed_text = {
        TextBlockContent::default()
            .add_line(
                TextLine::from("Hello World! ")
                    .add_item(TextItem::new("😊").with_font(noto_color_emoji.clone()))
                    .add_item(TextItem::new(" How are you?")),
            )
            .add_line(
                TextLine::from("More emoji: ")
                    .add_item(TextItem::new("👍").with_font(noto_color_emoji)),
            )
    };

    let text_block = TextBlock::from(mixed_text).with_style(TextStyle {
        font_ref: roboto,
        font_size: 24f32.pt(),
        ..Default::default()
    });

    page.add_to_layer(text_block.with_position((10f32.pt(), 800f32.pt()).into()))?;

    doc.add_page(page);

    let pdf = doc.write_into_pdf_document_writer()?;
    let mut file = std::fs::File::create(destination_dir().join("emoji_rasterize.pdf"))?;
    pdf.save(&mut file)?;

    Ok(())
}

/// Shows a simple imagge and some text
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
            ..Default::default()
        })
        .with_position(PdfPosition::new(10.0.pt(), 800.0.pt()));
    page.add_to_layer(text)?;

    let image = PdfImage::new(code_image_ref)
        .with_position(PdfPosition::new(10.0.pt(), 750.0.pt()))
        .with_scale(2f32, 2f32)
        .with_dpi(300.0);

    page.add_to_layer(image)?;

    doc.add_page(page);

    let pdf = doc.write_into_pdf_document_writer()?;

    let mut file = File::create(destination_dir().join("basic_image.pdf"))?;

    pdf.save(&mut file)?;

    Ok(())
}

/// Writes text with each possible roboto font
#[test]
fn all_roboto() -> anyhow::Result<()> {
    let mut doc = PdfDocument::new("Roboto Examples");
    test_utils::set_metadata_for_test(&mut doc);

    let mut page = PdfPage::new_from_page_size(A4);
    let mut text_position = PdfPosition {
        x: 10f32.into(),
        y: A4.top_left_point().y - 75f32.pt(),
    };
    for fonts in std::fs::read_dir(fonts_dir().join("Roboto"))? {
        let fonts = fonts?;
        if !does_end_with_ttf(fonts.path()) {
            continue;
        }
        let font_reader = std::fs::File::open(fonts.path())?;
        let read_font = OwnedPdfTtfFont::new_from_reader(font_reader, 0)?;
        let font = doc.font_map().register_external_font(read_font)?;
        let mut test_text = TextBlock {
            content: format!("Font is\n->{}", fonts.file_name().to_string_lossy()).into(),
            position: text_position,
            style: TextStyle {
                font_ref: font,
                font_size: 24f32.pt(),
                ..Default::default()
            },
            draw_as_lines: false,
        };
        let text_size = test_text.calculate_size(&doc)?;
        page.add_to_layer(test_text)?;
        text_position.y -= text_size.height;
    }

    doc.add_page(page);

    let pdf = doc.write_into_pdf_document_writer()?;
    let mut file = std::fs::File::create(destination_dir().join("all_roboto.pdf"))?;
    pdf.save(&mut file)?;

    Ok(())
}

/// Generates a PDF containing every glyph from NotoColorEmoji, rendered in a grid across multiple pages.
///
/// This test is ignored by default because it rasterizes ~3000 emoji glyphs which is slow in debug mode.
/// Run with: `cargo test --release all_noto_color_emojis -- --ignored`
#[test]
#[ignore]
fn all_noto_color_emojis() -> anyhow::Result<()> {
    init_logger();
    let mut doc = PdfDocument::new("All Noto Color Emojis");
    test_utils::set_metadata_for_test(&mut doc);

    // Enable rasterized emoji rendering for color output
    doc.set_emoji_render_mode(tux_pdf::document::EmojiRenderMode::rasterize_with_quality(
        64,
    ));

    let emoji_file = std::fs::File::open(
        fonts_dir()
            .join("Noto_Color_Emoji")
            .join("NotoColorEmoji-Regular.ttf"),
    )?;
    let emoji_font = OwnedPdfTtfFont::new_from_reader(emoji_file, 0)?;

    // Get all emoji characters from the font
    let glyph_map = emoji_font.glyph_ids();
    let mut emoji_chars: Vec<char> = glyph_map.values().copied().collect();
    emoji_chars.sort();

    println!(
        "NotoColorEmoji has {} mapped characters ({} total glyphs)",
        emoji_chars.len(),
        emoji_font.glyph_count()
    );

    let emoji_ref = doc
        .font_map()
        .register_external_font_with_name("NotoColorEmoji", emoji_font)?;

    // Also register Roboto for labels
    let roboto_file = std::fs::File::open(fonts_dir().join("Roboto").join("Roboto-Regular.ttf"))?;
    let roboto_font = OwnedPdfTtfFont::new_from_reader(roboto_file, 0)?;
    let roboto_ref = doc.font_map().register_external_font(roboto_font)?;

    // Layout parameters
    let font_size = 24f32;
    let cell_size = 32f32; // space per emoji
    let margin = 20f32;
    let page_width = A4.width;
    let page_height = A4.height;
    let cols = ((page_width.0 - 2.0 * margin) / cell_size) as usize;
    let rows_per_page = ((page_height.0 - 2.0 * margin - 30.0) / cell_size) as usize;
    let emojis_per_page = cols * rows_per_page;
    let mut number_of_pages = 0;
    for (page_idx, chunk) in emoji_chars.chunks(emojis_per_page).enumerate() {
        println!(
            "Rendering page {} with emojis {} to {}",
            page_idx + 1,
            chunk.first().unwrap_or(&' '),
            chunk.last().unwrap_or(&' ')
        );
        number_of_pages += 1;
        let mut page = PdfPage::new_from_page_size(A4);

        // Page header
        let header = TextBlock::from(format!(
            "Noto Color Emoji - Page {} of {} ({} emojis total)",
            page_idx + 1,
            emoji_chars.len().div_ceil(emojis_per_page),
            emoji_chars.len()
        ))
        .with_style(TextStyle {
            font_ref: roboto_ref.clone(),
            font_size: 12f32.pt(),
            ..Default::default()
        })
        .with_position(PdfPosition::new(margin.pt(), (page_height.0 - margin).pt()));
        page.add_to_layer(header)?;

        // Render emojis in a grid
        for (i, &ch) in chunk.iter().enumerate() {
            println!("Rendering emoji {} (char: {})", i, ch);
            let col = i % cols;
            let row = i / cols;
            let x = margin + (col as f32 * cell_size);
            // Start below the header (30pt gap)
            let y = page_height.0 - margin - 30.0 - (row as f32 * cell_size);

            if y < margin {
                break;
            }

            let emoji_text = TextBlock::from(TextBlockContent::from(
                TextLine::from("")
                    .add_item(TextItem::new(ch.to_string()).with_font(emoji_ref.clone())),
            ))
            .with_style(TextStyle {
                font_ref: emoji_ref.clone(),
                font_size: font_size.pt(),
                ..Default::default()
            })
            .with_position(PdfPosition::new(x.pt(), y.pt()));

            page.add_to_layer(emoji_text)?;
        }

        doc.add_page(page);
    }
    println!(
        "Finished adding {} emojis across {} pages",
        emoji_chars.len(),
        number_of_pages
    );
    let pdf = doc.write_into_pdf_document_writer()?;
    let mut file = std::fs::File::create(destination_dir().join("all_noto_color_emojis.pdf"))?;
    pdf.save(&mut file)?;

    println!(
        "Generated all_noto_color_emojis.pdf with {} pages",
        emoji_chars.len().div_ceil(emojis_per_page)
    );

    Ok(())
}
