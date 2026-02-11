use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::arguments::{Argument::Commit, CommitArgument},
    msg::Message,
};

pub fn keys(key: KeyEvent, arg_mode: bool) -> Option<Message> {
    if arg_mode {
        return match key.code {
            KeyCode::Char('a') => Some(Message::ToggleArgument(Commit(CommitArgument::StageAll))),
            // Any other key exits argument mode
            _ => Some(Message::ExitArgMode),
        };
    }
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('c') => Some(Message::Commit),
        KeyCode::Char('a') => Some(Message::Amend),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}
