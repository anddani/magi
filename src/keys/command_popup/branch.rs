use crossterm::event::{KeyCode, KeyEvent};

use crate::msg::Message;

pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('b') => Some(Message::ShowCheckoutBranchPopup),
        KeyCode::Char('x') => Some(Message::ShowDeleteBranchPopup),
        _ => None,
    }
}
