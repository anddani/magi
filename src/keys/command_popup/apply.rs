use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::popup::ApplyPopupState,
    msg::{ApplyCommand, Message},
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

    let has_commits = !state.selected_commits.is_empty();
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('A') if has_commits => Some(Message::Apply(ApplyCommand::Pick(
            state.selected_commits.clone(),
        ))),
        _ => None,
    }
}
