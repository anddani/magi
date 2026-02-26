use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::arguments::{Argument::Stash, StashArgument},
    msg::{Message, SelectPopup, StashType},
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
        KeyCode::Char('z') => Some(Message::ShowStashInput(StashType::Both)),
        KeyCode::Char('i') => Some(Message::ShowStashInput(StashType::Index)),
        KeyCode::Char('w') => Some(Message::ShowStashInput(StashType::Workspace)),
        KeyCode::Char('a') => Some(Message::ShowSelectPopup(SelectPopup::StashApply)),
        KeyCode::Char('p') => Some(Message::ShowSelectPopup(SelectPopup::StashPop)),
        KeyCode::Char('k') => Some(Message::ShowSelectPopup(SelectPopup::StashDrop)),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}
