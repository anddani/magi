use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::msg::{Message, SelectMessage};

pub fn keys(key: KeyEvent) -> Option<Message> {
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Enter) => Some(Message::Select(SelectMessage::Confirm)),
        (KeyModifiers::NONE, KeyCode::Backspace) => {
            Some(Message::Select(SelectMessage::InputBackspace))
        }
        (KeyModifiers::NONE, KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
            Some(Message::Select(SelectMessage::MoveUp))
        }
        (KeyModifiers::NONE, KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
            Some(Message::Select(SelectMessage::MoveDown))
        }
        (KeyModifiers::NONE, KeyCode::Char(c)) | (KeyModifiers::SHIFT, KeyCode::Char(c)) => {
            Some(Message::Select(SelectMessage::InputChar(c)))
        }
        _ => None,
    }
}
