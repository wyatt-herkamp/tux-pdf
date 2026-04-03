//! Emoji glyph rasterizer — converts color font glyphs (SVG/CBDT/sbix) into PDF images.

use crate::{TuxPdfError, document::PdfXObjectImage};

use super::ExternalLoadedFont;

/// Rasterize multiple emoji glyphs from a font, returning results in the same order as input.
///
/// This is much faster than calling `rasterize_glyph` in a loop because it caches
/// parsed SVG trees — NotoColorEmoji shares one SVG document across many glyph IDs.
pub(crate) fn rasterize_glyphs(
    font: &dyn ExternalLoadedFont,
    glyph_ids: &[u16],
    pixels_per_em: u16,
) -> Vec<Result<PdfXObjectImage, TuxPdfError>> {
    #[cfg(feature = "svg")]
    {
        rasterize_glyphs_with_svg_cache(font, glyph_ids, pixels_per_em)
    }
    #[cfg(not(feature = "svg"))]
    {
        glyph_ids
            .iter()
            .map(|&gid| rasterize_glyph(font, gid, pixels_per_em))
            .collect()
    }
}

/// Rasterize a single emoji glyph from a font into a `PdfXObjectImage`.
#[allow(dead_code)]
pub(crate) fn rasterize_glyph(
    font: &dyn ExternalLoadedFont,
    glyph_id: u16,
    pixels_per_em: u16,
) -> Result<PdfXObjectImage, TuxPdfError> {
    // Try SVG path first
    if let Some(svg_data) = font.glyph_svg_data(glyph_id) {
        return rasterize_svg_glyph(svg_data, glyph_id, pixels_per_em, font);
    }

    // Try raster path (CBDT/sbix)
    if let Some(raster) = font.glyph_raster_data(glyph_id, pixels_per_em) {
        return rasterize_bitmap_glyph(&raster);
    }

    Err(TuxPdfError::InvalidObjectId(format!(
        "No color glyph data found for glyph ID {glyph_id}"
    )))
}

#[cfg(feature = "svg")]
fn rasterize_glyphs_with_svg_cache(
    font: &dyn ExternalLoadedFont,
    glyph_ids: &[u16],
    pixels_per_em: u16,
) -> Vec<Result<PdfXObjectImage, TuxPdfError>> {
    glyph_ids
        .iter()
        .map(|&glyph_id| {
            // Try SVG path first
            if let Some(svg_data) = font.glyph_svg_data(glyph_id) {
                let tree = parse_svg_tree_for_glyph(svg_data, glyph_id, font)?;
                return render_svg_tree_to_image(&tree, pixels_per_em);
            }

            // Try raster path (CBDT/sbix)
            if let Some(raster) = font.glyph_raster_data(glyph_id, pixels_per_em) {
                return rasterize_bitmap_glyph(&raster);
            }

            Err(TuxPdfError::InvalidObjectId(format!(
                "No color glyph data found for glyph ID {glyph_id}"
            )))
        })
        .collect()
}

fn rasterize_svg_glyph(
    svg_data: &[u8],
    glyph_id: u16,
    pixels_per_em: u16,
    font: &dyn ExternalLoadedFont,
) -> Result<PdfXObjectImage, TuxPdfError> {
    #[cfg(feature = "svg")]
    {
        let tree = parse_svg_tree_for_glyph(svg_data, glyph_id, font)?;
        render_svg_tree_to_image(&tree, pixels_per_em)
    }
    #[cfg(not(feature = "svg"))]
    {
        let _ = (svg_data, glyph_id, pixels_per_em, font);
        tracing::warn!(
            "Emoji font has SVG glyphs but the 'svg' feature is not enabled. \
             Falling back to EmbedFont mode."
        );
        Err(TuxPdfError::InvalidObjectId(
            "SVG emoji rasterization requires the 'svg' feature".to_string(),
        ))
    }
}

#[cfg(feature = "svg")]
fn parse_svg_tree_for_glyph(
    svg_data: &[u8],
    glyph_id: u16,
    font: &dyn ExternalLoadedFont,
) -> Result<resvg::usvg::Tree, TuxPdfError> {
    let svg_str = std::str::from_utf8(svg_data)
        .map_err(|e| TuxPdfError::InvalidObjectId(format!("Invalid UTF-8 in SVG glyph: {e}")))?;

    // NotoColorEmoji SVGs use font coordinate space where Y=0 is the baseline,
    // negative Y is above baseline (ascender direction), and positive Y is below
    // (descender direction). The SVGs have <svg> root elements but no viewBox,
    // so we must inject one covering the full ascender-to-descender range.
    let ascender = font.ascender() as i32;
    let descender = font.descender() as i32;
    let em = font.units_per_em() as i32;
    let vb_y = -ascender;
    let vb_height = ascender - descender;
    let viewbox = format!("0 {vb_y} {em} {vb_height}");

    // NotoColorEmoji SVGs can contain multiple glyphs as <g id="glyphN"> groups.
    // Hide all glyph groups except the one we want by adding display="none".
    let target_id = format!("glyph{glyph_id}");
    let mut svg_doc = if svg_str.trim_start().starts_with("<svg") {
        svg_str.replacen(
            "<svg",
            &format!(r#"<svg viewBox="{viewbox}" width="{em}" height="{vb_height}""#),
            1,
        )
    } else {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{viewbox}" width="{em}" height="{vb_height}">{svg_str}</svg>"#,
        )
    };

    // Hide non-target glyph groups. NotoColorEmoji SVGs use `<g id="glyphN">`
    // for each glyph. Add display="none" to all groups except the target.
    let search_pattern = r#"<g id="glyph"#;
    if svg_doc.contains(search_pattern) {
        let target_tag = format!(r#"<g id="{target_id}""#);
        let mut result = String::with_capacity(svg_doc.len() + 256);
        let mut remaining = svg_doc.as_str();
        while let Some(pos) = remaining.find(r#"<g id="glyph"#) {
            result.push_str(&remaining[..pos]);
            // Find the end of this opening tag attribute: the closing `"`  after the glyph ID
            let after_prefix = &remaining[pos..];
            if after_prefix.starts_with(&target_tag) {
                // Keep the target glyph visible
                let end = after_prefix.find('"').unwrap(); // first quote (in `<g id="`)
                let end = after_prefix[end + 1..].find('"').unwrap() + end + 1 + 1; // past closing quote
                result.push_str(&after_prefix[..end]);
                remaining = &after_prefix[end..];
            } else {
                // Hide this non-target glyph group
                // Find the id value: <g id="glyphNNN"
                let id_start = pos + r#"<g id=""#.len();
                let id_end_in_remaining = remaining[id_start..].find('"').unwrap() + id_start;
                let id_val = &remaining[id_start..id_end_in_remaining];
                result.push_str(&format!(r#"<g id="{id_val}" display="none""#));
                remaining = &remaining[id_end_in_remaining + 1..]; // skip past closing quote
            }
        }
        result.push_str(remaining);
        svg_doc = result;
    }

    let opts = resvg::usvg::Options::default();
    resvg::usvg::Tree::from_str(&svg_doc, &opts)
        .map_err(|e| TuxPdfError::InvalidObjectId(format!("Failed to parse SVG glyph: {e}")))
}

#[cfg(feature = "svg")]
fn render_svg_tree_to_image(
    tree: &resvg::usvg::Tree,
    pixels_per_em: u16,
) -> Result<PdfXObjectImage, TuxPdfError> {
    let tree_size = tree.size();

    // Create a pixmap that matches the SVG's aspect ratio to avoid distortion.
    // Scale uniformly so the longest axis fits pixels_per_em.
    let scale = pixels_per_em as f32 / tree_size.width().max(tree_size.height());
    let px_width = (tree_size.width() * scale).ceil() as u32;
    let px_height = (tree_size.height() * scale).ceil() as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(px_width, px_height).ok_or_else(|| {
        TuxPdfError::InvalidObjectId("Failed to create pixmap for emoji".to_string())
    })?;

    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(tree, transform, &mut pixmap.as_mut());

    pixmap_to_pdf_image(pixmap)
}

#[cfg(feature = "svg")]
fn pixmap_to_pdf_image(pixmap: resvg::tiny_skia::Pixmap) -> Result<PdfXObjectImage, TuxPdfError> {
    use crate::document::PdfXObjectImageData;
    use crate::graphics::color::{ColorBits, ColorSpace};
    use crate::graphics::size::Size;
    use crate::units::Px;

    let width = pixmap.width();
    let height = pixmap.height();
    let pixels = pixmap.data();

    // tiny_skia uses premultiplied alpha. We need to unpremultiply for PDF.
    let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
    let mut alpha_data = Vec::with_capacity((width * height) as usize);

    for pixel in pixels.chunks_exact(4) {
        let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
        if a == 0 {
            rgb_data.extend_from_slice(&[255, 255, 255]);
            alpha_data.push(0);
        } else {
            // Unpremultiply
            let r = ((r as u16 * 255) / a as u16) as u8;
            let g = ((g as u16 * 255) / a as u16) as u8;
            let b = ((b as u16 * 255) / a as u16) as u8;
            rgb_data.extend_from_slice(&[r, g, b]);
            alpha_data.push(a);
        }
    }

    let has_alpha = alpha_data.iter().any(|&a| a != 255);

    let image_data = PdfXObjectImageData {
        size: Size {
            width: Px(width as i64),
            height: Px(height as i64),
        },
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        image_data: rgb_data,
        image_filter: None,
        interpolate: true,
        smask: None,
        clipping_bbox: None,
    };

    if has_alpha {
        let mask_data = PdfXObjectImageData {
            size: Size {
                width: Px(width as i64),
                height: Px(height as i64),
            },
            color_space: ColorSpace::Greyscale,
            bits_per_component: ColorBits::Bit8,
            image_data: alpha_data,
            image_filter: None,
            interpolate: true,
            smask: None,
            clipping_bbox: None,
        };
        Ok(PdfXObjectImage {
            image: image_data,
            mask: Some(mask_data),
        })
    } else {
        Ok(PdfXObjectImage {
            image: image_data,
            mask: None,
        })
    }
}

fn rasterize_bitmap_glyph(
    raster: &ttf_parser::RasterGlyphImage<'_>,
) -> Result<PdfXObjectImage, TuxPdfError> {
    // CBDT/sbix typically stores PNG data.
    // Use ImageReader with format guessing since specific decoders may not be compiled in.
    let cursor = std::io::Cursor::new(raster.data);
    let reader = image::ImageReader::new(cursor)
        .with_guessed_format()
        .map_err(|e| {
            TuxPdfError::InvalidObjectId(format!("Failed to guess raster emoji format: {e}"))
        })?;
    let dynamic_image = reader.decode()?;
    PdfXObjectImage::load_from_dynamic_image(dynamic_image)
}
