use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::popup::RebasePopupState,
    msg::{Message, RebaseCommand, SelectPopup},
};

pub fn keys(key: KeyEvent, state: &RebasePopupState) -> Option<Message> {
    if state.in_progress {
        return match key.code {
            KeyCode::Char('q') => Some(Message::DismissPopup),
            KeyCode::Char('r') => Some(Message::Rebase(RebaseCommand::Continue)),
            KeyCode::Char('s') => Some(Message::Rebase(RebaseCommand::Skip)),
            KeyCode::Char('a') => Some(Message::Rebase(RebaseCommand::Abort)),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('e') => Some(Message::ShowSelectPopup(SelectPopup::RebaseElsewhere)),
        _ => None,
    }
}
