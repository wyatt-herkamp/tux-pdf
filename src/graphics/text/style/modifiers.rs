use std::borrow::Cow;

use crate::{
    document::FontRef,
    graphics::{state::TextBlockState, OperationWriter, TextOperations},
    units::Pt,
    TuxPdfError,
};

#[derive(Debug, Clone, PartialEq)]
pub enum TextModifier {
    FontSize(Pt),
    Font(FontRef),
    TextRise(Pt),
    CharacterSpacing(Pt),
    WordSpacing(Pt),
}

pub(crate) fn write_modifiers<'state, 'resources>(
    modifiers: Vec<TextModifier>,
    current_state: &'state TextBlockState<'resources>,
    writer: &mut OperationWriter,
) -> Result<Cow<'state, TextBlockState<'resources>>, TuxPdfError> {
    if modifiers.is_empty() {
        return Ok(Cow::Borrowed(current_state));
    }
    let mut updating_state = current_state.create_updating();
    for modifier in modifiers {
        match modifier {
            TextModifier::FontSize(size) => {
                updating_state.font_size = Some(size);
            }
            TextModifier::Font(font_ref) => {
                updating_state.font = Some(font_ref);
            }
            TextModifier::TextRise(rise) => {
                updating_state.text_rise = Some(rise);
                writer.add_operation(TextOperations::TextRise, vec![rise.into()]);
            }
            TextModifier::CharacterSpacing(spacing) => {
                updating_state.character_spacing = Some(spacing);
                writer.add_operation(TextOperations::CharacterSpace, vec![spacing.into()]);
            }
            TextModifier::WordSpacing(spacing) => {
                updating_state.word_spacing = Some(spacing);
                writer.add_operation(TextOperations::WordSpace, vec![spacing.into()]);
            }
        }
    }
    if let Some(new_state) = updating_state.build(Some(writer))? {
        Ok(Cow::Owned(new_state))
    } else {
        Ok(Cow::Borrowed(current_state))
    }
}

pub(crate) fn state_from_modifiers<'state, 'resources>(
    modifiers: &[TextModifier],
    current_state: &'state TextBlockState<'resources>,
) -> Result<Cow<'state, TextBlockState<'resources>>, TuxPdfError> {
    if modifiers.is_empty() {
        return Ok(Cow::Borrowed(current_state));
    }
    let mut block_state = current_state.create_updating();
    for modifier in modifiers {
        match modifier {
            TextModifier::FontSize(size) => {
                block_state.font_size = Some(*size);
            }
            TextModifier::Font(font_ref) => {
                block_state.font = Some(font_ref.clone());
            }
            TextModifier::TextRise(rise) => {
                block_state.text_rise = Some(*rise);
            }
            TextModifier::CharacterSpacing(spacing) => {
                block_state.character_spacing = Some(*spacing);
            }
            TextModifier::WordSpacing(spacing) => {
                block_state.word_spacing = Some(*spacing);
            }
        }
    }
    if let Some(new_state) = block_state.build(None)? {
        Ok(Cow::Owned(new_state))
    } else {
        Ok(Cow::Borrowed(current_state))
    }
}
