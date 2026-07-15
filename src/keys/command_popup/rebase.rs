use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::popup::RebasePopupState,
    msg::{CommitSelect, Message, OnSelect, OptionsSource, RebaseCommand, ShowSelectPopupConfig},
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
        KeyCode::Char('p') => {
            if let Some(remote) = state.push_remote.as_ref().or(state.sole_remote.as_ref()) {
                Some(Message::Rebase(RebaseCommand::OntoPushRemote(
                    remote.clone(),
                )))
            } else {
                Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                    title: "Rebase onto push remote".to_string(),
                    source: OptionsSource::Remotes,
                    on_select: OnSelect::RebasePushRemote,
                }))
            }
        }
        KeyCode::Char('e') => Some(Message::ShowCommitSelect(CommitSelect::RebaseElsewhere)),
        KeyCode::Char('i') => Some(Message::ShowCommitSelect(CommitSelect::RebaseInteractive)),
        _ => None,
    }
}
