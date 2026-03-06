use crossterm::event::{KeyCode, KeyEvent};

use crate::msg::{Message, ResetMode, SelectPopup};

pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('b') => Some(Message::ShowSelectPopup(SelectPopup::ResetBranchPick)),
        KeyCode::Char('f') => Some(Message::ShowSelectPopup(SelectPopup::FileCheckoutRevision)),
        KeyCode::Char('m') => Some(Message::ShowSelectPopup(SelectPopup::Reset(
            ResetMode::Mixed,
        ))),
        KeyCode::Char('s') => Some(Message::ShowSelectPopup(SelectPopup::Reset(
            ResetMode::Soft,
        ))),
        _ => None,
    }
}
