//! Handler for credential popup confirmation.

use crate::git::credential::CredentialResponse;
use crate::model::{popup::CredentialPopupState, Model, PopupContent};
use crate::msg::Message;

/// Handles Enter being pressed in the credential popup.
/// Sends the credential to the PTY thread.
pub fn update(model: &mut Model) -> Option<Message> {
    // Get the input text from the popup
    let input = if let Some(PopupContent::Credential(CredentialPopupState { input_text, .. })) =
        &model.popup
    {
        input_text.clone()
    } else {
        return None;
    };

    // Send credential to PTY thread
    if let Some(ref pty_state) = model.pty_state {
        let _ = pty_state.send_credential(CredentialResponse::Input(input));
    }

    // Clear the popup
    model.popup = None;

    None
}
