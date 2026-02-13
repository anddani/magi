use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::msg::{InputMessage, Message};

pub fn handle_input_popup_key(key: KeyEvent) -> Option<Message> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('g')) => {
            Some(Message::DismissPopup)
        }
        (_, KeyCode::Enter) => Some(Message::Input(InputMessage::Confirm)),
        (_, KeyCode::Backspace) => Some(Message::Input(InputMessage::InputBackspace)),
        (_, KeyCode::Char(c)) => Some(Message::Input(InputMessage::InputChar(c))),
        _ => None,
    }
}
