use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::popup::MergePopupState,
    msg::{MergeCommand, Message, OnSelect, OptionsSource, ShowSelectPopupConfig},
};

pub fn keys(key: KeyEvent, state: &MergePopupState) -> Option<Message> {
    if state.in_progress {
        return match key.code {
            KeyCode::Char('q') => Some(Message::DismissPopup),
            KeyCode::Char('m') => Some(Message::Merge(MergeCommand::Continue)),
            KeyCode::Char('a') => Some(Message::Merge(MergeCommand::Abort)),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('m') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Merge branch".to_string(),
            source: OptionsSource::LocalAndRemoteBranches,
            on_select: OnSelect::MergeElsewhere,
        })),
        _ => None,
    }
}
