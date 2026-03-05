use crossterm::event::{KeyCode, KeyEvent};

use crate::msg::{Message, SelectPopup};

pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('b') => Some(Message::ShowSelectPopup(SelectPopup::ResetBranchPick)),
        KeyCode::Char('f') => Some(Message::ShowSelectPopup(SelectPopup::FileCheckoutRevision)),
        _ => None,
    }
}
