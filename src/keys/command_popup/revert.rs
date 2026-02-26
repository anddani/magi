use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::popup::RevertPopupState,
    msg::{Message, RevertCommand},
};

pub fn keys(key: KeyEvent, state: &RevertPopupState) -> Option<Message> {
    if state.in_progress {
        return match key.code {
            KeyCode::Char('q') => Some(Message::DismissPopup),
            KeyCode::Char('_') => Some(Message::Revert(RevertCommand::Continue)),
            KeyCode::Char('s') => Some(Message::Revert(RevertCommand::Skip)),
            KeyCode::Char('a') => Some(Message::Revert(RevertCommand::Abort)),
            _ => None,
        };
    }

    let has_commits = !state.selected_commits.is_empty();
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('_') if has_commits => Some(Message::Revert(RevertCommand::Commits(
            state.selected_commits.clone(),
        ))),
        KeyCode::Char('v') if has_commits => Some(Message::Revert(RevertCommand::NoCommit(
            state.selected_commits.clone(),
        ))),
        _ => None,
    }
}
