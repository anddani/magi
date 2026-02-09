use crossterm::event::{KeyCode, KeyEvent};

use crate::msg::Message;

pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('a') => Some(Message::FetchAllRemotes),
        _ => None,
    }
}
