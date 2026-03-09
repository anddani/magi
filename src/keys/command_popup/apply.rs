use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::popup::ApplyPopupState,
    msg::{ApplyCommand, Message, SelectPopup},
};

pub fn keys(key: KeyEvent, state: &ApplyPopupState) -> Option<Message> {
    if state.in_progress {
        return match key.code {
            KeyCode::Char('q') => Some(Message::DismissPopup),
            KeyCode::Char('A') => Some(Message::Apply(ApplyCommand::Continue)),
            KeyCode::Char('s') => Some(Message::Apply(ApplyCommand::Skip)),
            KeyCode::Char('a') => Some(Message::Apply(ApplyCommand::Abort)),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('A') => Some(Message::ShowSelectPopup(SelectPopup::ApplyPick)),
        _ => None,
    }
}
