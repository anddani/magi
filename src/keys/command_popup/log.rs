use crossterm::event::{KeyCode, KeyEvent};

use crate::msg::Message;

/// Handle key events for the Log command popup
pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('l') => Some(Message::ShowLogCurrent),
        _ => None,
    }
}
