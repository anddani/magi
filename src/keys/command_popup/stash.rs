use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::arguments::{Argument::Stash, StashArgument},
    msg::{Message, ShowSelectDialog},
};

pub fn keys(key: KeyEvent, arg_mode: bool) -> Option<Message> {
    if arg_mode {
        return match key.code {
            KeyCode::Char(c) => StashArgument::from_key(c)
                .map(|arg| Message::ToggleArgument(Stash(arg)))
                .or(Some(Message::ExitArgMode)),
            _ => Some(Message::ExitArgMode),
        };
    }

    match key.code {
        KeyCode::Char('z') => Some(Message::ShowStashMessageInput),
        KeyCode::Char('a') => Some(Message::ShowSelect(ShowSelectDialog::StashApply)),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}
