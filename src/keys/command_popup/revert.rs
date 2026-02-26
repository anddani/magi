use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::popup::RevertPopupState,
    msg::{Message, RevertCommand},
};

pub fn keys(key: KeyEvent, state: &RevertPopupState) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('_') => {
            if state.in_progress {
                Some(Message::Revert(RevertCommand::Continue))
            } else if !state.selected_commits.is_empty() {
                Some(Message::Revert(RevertCommand::Commits(
                    state.selected_commits.clone(),
                )))
            } else {
                None
            }
        }
        KeyCode::Char('s') if state.in_progress => Some(Message::Revert(RevertCommand::Skip)),
        KeyCode::Char('a') if state.in_progress => Some(Message::Revert(RevertCommand::Abort)),
        _ => None,
    }
}
