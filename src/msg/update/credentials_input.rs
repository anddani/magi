//! Handler for credential popup input event.

use crate::git::credential::CredentialResponse;
use crate::model::{Model, PopupContent, popup::CredentialPopupState};
use crate::msg::{CredentialsMessage, Message};

/// Handles a character being typed in the credential popup.
pub fn update(model: &mut Model, credentials_msg: CredentialsMessage) -> Option<Message> {
    let PopupContent::Credential(CredentialPopupState { input, .. }) = model.popup.as_mut()? else {
        return None;
    };
    match credentials_msg {
        CredentialsMessage::Edit(op) => {
            input.apply(op);
        }
        CredentialsMessage::CredentialConfirm => {
            // Get the input text from the popup
            let input = input.as_str().to_string();

            // Send credential to PTY thread
            if let Some(ref pty_state) = model.pty_state {
                let _ = pty_state.send_credential(CredentialResponse::Input(input));
            }

            // Clear the popup
            model.popup = None;
        }
    }
    None
}
