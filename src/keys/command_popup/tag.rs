use crossterm::event::{KeyCode, KeyEvent};

use crate::{model::popup::TagPopupState, msg::Message};

pub fn keys(key: KeyEvent, _state: &TagPopupState) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        _ => None,
    }
}
