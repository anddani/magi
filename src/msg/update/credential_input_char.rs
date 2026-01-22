//! Handler for credential popup character input.

use crate::model::{popup::CredentialPopupState, Model, PopupContent};
use crate::msg::Message;

/// Handles a character being typed in the credential popup.
pub fn update(model: &mut Model, c: char) -> Option<Message> {
    if let Some(PopupContent::Credential(CredentialPopupState { input_text, .. })) =
        &mut model.popup
    {
        input_text.push(c);
    }
    None
}
