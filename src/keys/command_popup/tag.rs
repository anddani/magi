use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::popup::TagPopupState,
    msg::{Message, SelectPopup},
};

pub fn keys(key: KeyEvent, _state: &TagPopupState) -> Option<Message> {
    match key.code {
        KeyCode::Char('t') => Some(Message::ShowCreateTagInput),
        KeyCode::Char('x') => Some(Message::ShowSelectPopup(SelectPopup::DeleteTag)),
        KeyCode::Char('q') => Some(Message::DismissPopup),
        _ => None,
    }
}
