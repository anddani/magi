use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::arguments::{Argument::Tag, TagArgument},
    msg::{Message, OnSelect, OptionsSource, ShowSelectPopupConfig},
};

pub fn keys(key: KeyEvent, arg_mode: bool) -> Option<Message> {
    if arg_mode {
        return match key.code {
            KeyCode::Char(c) => TagArgument::from_key(c)
                .map(|arg| Message::ToggleArgument(Tag(arg)))
                .or(Some(Message::ExitArgMode)),
            _ => Some(Message::ExitArgMode),
        };
    }

    match key.code {
        KeyCode::Char('t') => Some(Message::ShowCreateTagInput),
        KeyCode::Char('r') => Some(Message::ShowTagReleaseInput),
        KeyCode::Char('x') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Delete tag".to_string(),
            source: OptionsSource::Tags,
            on_select: OnSelect::DeleteTag,
        })),
        KeyCode::Char('p') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Prune tags against".to_string(),
            source: OptionsSource::Remotes,
            on_select: OnSelect::PruneTagsRemotePick,
        })),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        KeyCode::Char('q') => Some(Message::DismissPopup),
        _ => None,
    }
}
