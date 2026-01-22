//! Handler for credential popup backspace.

use crate::model::{popup::CredentialPopupState, Model, PopupContent};
use crate::msg::Message;

/// Handles backspace in the credential popup.
pub fn update(model: &mut Model) -> Option<Message> {
    if let Some(PopupContent::Credential(CredentialPopupState { input_text, .. })) =
        &mut model.popup
    {
        input_text.pop();
    }
    None
}
