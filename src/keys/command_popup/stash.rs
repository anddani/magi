use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::arguments::{Argument::Stash, StashArgument},
    msg::{Message, OnSelect, OptionsSource, ShowSelectPopupConfig, StashType},
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
        KeyCode::Char('w') => Some(Message::ShowStashInput(StashType::Worktree)),
        KeyCode::Char('a') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Apply stash".to_string(),
            source: OptionsSource::Stashes,
            on_select: OnSelect::ApplyStash,
        })),
        KeyCode::Char('p') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Pop stash".to_string(),
            source: OptionsSource::Stashes,
            on_select: OnSelect::PopStash,
        })),
        KeyCode::Char('k') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Drop stash".to_string(),
            source: OptionsSource::Stashes,
            on_select: OnSelect::DropStash,
        })),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}
