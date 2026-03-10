use crossterm::event::{KeyCode, KeyEvent};

use crate::msg::{Message, OnSelect, OptionsSource, ShowSelectPopupConfig};

pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('t') => Some(Message::ShowCreateTagInput),
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
        KeyCode::Char('q') => Some(Message::DismissPopup),
        _ => None,
    }
}
