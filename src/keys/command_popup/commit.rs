use crossterm::event::{KeyCode, KeyEvent};

use crate::msg::Message;

pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('c') => Some(Message::Commit),
        KeyCode::Char('a') => Some(Message::Amend),
        _ => None,
    }
}
