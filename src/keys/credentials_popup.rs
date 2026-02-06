use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::msg::{CredentialsMessage, Message};

pub fn handle_credentials_popup_key(key: KeyEvent) -> Option<Message> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('g')) => {
            Some(Message::DismissPopup)
        }
        (_, KeyCode::Enter) => Some(Message::Credentials(CredentialsMessage::CredentialConfirm)),
        (_, KeyCode::Backspace) => Some(Message::Credentials(
            CredentialsMessage::CredentialInputBackspace,
        )),
        (_, KeyCode::Char(c)) => Some(Message::Credentials(
            CredentialsMessage::CredentialInputChar(c),
        )),
        _ => None,
    }
}
