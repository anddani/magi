use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::keys::edit::edit_op_for_key;
use crate::msg::{InputMessage, Message};

pub fn handle_input_popup_key(key: KeyEvent) -> Option<Message> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('g')) => {
            Some(Message::DismissPopup)
        }
        (_, KeyCode::Enter) => Some(Message::Input(InputMessage::Confirm)),
        _ => edit_op_for_key(key).map(|op| Message::Input(InputMessage::Edit(op))),
    }
}
