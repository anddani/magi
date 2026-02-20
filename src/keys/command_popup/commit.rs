use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::arguments::{Argument::Commit, CommitArgument},
    msg::Message,
};

pub fn keys(key: KeyEvent, arg_mode: bool) -> Option<Message> {
    if arg_mode {
        return match key.code {
            KeyCode::Char(c) => CommitArgument::from_key(c)
                .map(|arg| Message::ToggleArgument(Commit(arg)))
                .or(Some(Message::ExitArgMode)),
            _ => Some(Message::ExitArgMode),
        };
    }
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('c') => Some(Message::Commit),
        KeyCode::Char('e') => Some(Message::Amend(vec!["--no-edit".to_string()])),
        KeyCode::Char('a') => Some(Message::Amend(vec![])),
        KeyCode::Char('w') => Some(Message::Amend(vec!["--only".to_string()])),
        KeyCode::Char('f') => Some(Message::ShowFixupCommitSelect),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}
