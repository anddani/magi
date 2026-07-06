use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::keys::edit::edit_op_for_key;
use crate::msg::{Message, SelectMessage};

pub fn keys(key: KeyEvent) -> Option<Message> {
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Enter) => Some(Message::Select(SelectMessage::Confirm)),
        (KeyModifiers::NONE, KeyCode::Up) | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
            Some(Message::Select(SelectMessage::MoveUp))
        }
        (KeyModifiers::NONE, KeyCode::Down) | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
            Some(Message::Select(SelectMessage::MoveDown))
        }
        _ => edit_op_for_key(key).map(|op| Message::Select(SelectMessage::Edit(op))),
    }
}
