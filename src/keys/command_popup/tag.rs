use crossterm::event::{KeyCode, KeyEvent};

use crate::msg::{Message, SelectPopup};

pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('t') => Some(Message::ShowCreateTagInput),
        KeyCode::Char('x') => Some(Message::ShowSelectPopup(SelectPopup::DeleteTag)),
        KeyCode::Char('p') => Some(Message::ShowSelectPopup(SelectPopup::PruneTagsRemotePick)),
        KeyCode::Char('q') => Some(Message::DismissPopup),
        _ => None,
    }
}
