use crate::{
    document::{FontRef, PdfResources},
    graphics::{OperationKeys, OperationWriter, Point, TextBlockState},
    units::Pt,
    TuxPdfError,
};

#[derive(Debug, Clone, PartialEq)]
pub enum TextModifier {
    FontSize(Pt),
    Font(FontRef),
    Position(Point),
}

pub(crate) fn write_modifiers<'resources>(
    modifiers: Vec<TextModifier>,
    current_state: &TextBlockState<'resources>,
    resources: &'resources PdfResources,
    writer: &mut OperationWriter,
) -> Result<Option<TextBlockState<'resources>>, TuxPdfError> {
    if modifiers.is_empty() {
        return Ok(None);
    }
    let mut font_size = None;
    let mut font = None;
    for modifier in modifiers {
        match modifier {
            TextModifier::FontSize(size) => {
                font_size = Some(size);
            }
            TextModifier::Font(font_ref) => {
                font = Some(font_ref);
            }
            TextModifier::Position(position) => {
                writer.add_operation(OperationKeys::TextPosition, position.into());
            }
        }
    }
    let block_state = match (font_size, font) {
        (Some(size), Some(font)) => {
            let font_type = resources
                .fonts
                .internal_font_type(&font)
                .ok_or(TuxPdfError::from(font.clone()))?;
            let new_state = TextBlockState {
                font: font.clone(),
                font_size: size,
                font_type,
            };
            writer.add_operation(OperationKeys::TextFont, vec![font.into(), size.into()]);
            Some(new_state)
        }
        (Some(size), None) => {
            writer.add_operation(
                OperationKeys::TextFont,
                vec![current_state.font.clone().into(), size.into()],
            );
            Some(TextBlockState {
                font: current_state.font.clone(),
                font_size: size,
                font_type: current_state.font_type,
            })
        }
        (None, Some(font)) => {
            let font_type = resources
                .fonts
                .internal_font_type(&font)
                .ok_or(TuxPdfError::from(font.clone()))?;
            let new_state = TextBlockState {
                font: font.clone(),
                font_size: current_state.font_size,
                font_type,
            };
            writer.add_operation(
                OperationKeys::TextFont,
                vec![font.into(), current_state.font_size.into()],
            );
            Some(new_state)
        }
        _ => None,
    };
    Ok(block_state)
}

pub(crate) fn state_from_modifiers<'resources>(
    modifiers: &[TextModifier],
    current_state: &TextBlockState<'resources>,
    resources: &'resources PdfResources,
) -> Result<Option<TextBlockState<'resources>>, TuxPdfError> {
    if modifiers.is_empty() {
        return Ok(None);
    }
    let mut font_size = None;
    let mut font = None;
    for modifier in modifiers {
        match modifier {
            TextModifier::FontSize(size) => {
                font_size = Some(size);
            }
            TextModifier::Font(font_ref) => {
                font = Some(font_ref);
            }
            _ => {}
        }
    }
    let block_state = match (font_size, font) {
        (Some(size), Some(font)) => {
            let font_type = resources
                .fonts
                .internal_font_type(&font)
                .ok_or(TuxPdfError::from(font.clone()))?;
            let new_state = TextBlockState {
                font: font.clone(),
                font_size: *size,
                font_type,
            };

            Some(new_state)
        }
        (Some(size), None) => Some(TextBlockState {
            font: current_state.font.clone(),
            font_size: *size,
            font_type: current_state.font_type,
        }),
        (None, Some(font)) => {
            let font_type = resources
                .fonts
                .internal_font_type(&font)
                .ok_or(TuxPdfError::from(font.clone()))?;
            let new_state = TextBlockState {
                font: font.clone(),
                font_size: current_state.font_size,
                font_type,
            };
            Some(new_state)
        }
        _ => None,
    };
    Ok(block_state)
}
