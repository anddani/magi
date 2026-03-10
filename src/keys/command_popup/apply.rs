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
        KeyCode::Char('A') => {
            if state.selected_commits.is_empty() {
                Some(Message::ShowSelectPopup(SelectPopup::ApplyPick))
            } else {
                Some(Message::Apply(ApplyCommand::Pick(
                    state.selected_commits.clone(),
                )))
            }
        }
        KeyCode::Char('a') => {
            if state.selected_commits.is_empty() {
                Some(Message::ShowSelectPopup(SelectPopup::ApplyApply))
            } else {
                Some(Message::Apply(ApplyCommand::Apply(
                    state.selected_commits.clone(),
                )))
            }
        }
        _ => None,
    }
}
