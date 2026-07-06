use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::keys::edit::edit_op_for_key;
use crate::msg::{CredentialsMessage, Message};

pub fn handle_credentials_popup_key(key: KeyEvent) -> Option<Message> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('g')) => {
            Some(Message::DismissPopup)
        }
        (_, KeyCode::Enter) => Some(Message::Credentials(CredentialsMessage::CredentialConfirm)),
        _ => edit_op_for_key(key).map(|op| Message::Credentials(CredentialsMessage::Edit(op))),
    }
}
