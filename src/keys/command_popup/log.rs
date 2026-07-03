use crossterm::event::{KeyCode, KeyEvent};

use crate::msg::{LogType, Message, OnSelect, OptionsSource, ShowSelectPopupConfig};

/// Handle key events for the Log command popup
pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('l') => Some(Message::ShowLog(LogType::Current)),
        KeyCode::Char('o') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Log revision".to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::LogOther,
        })),
        KeyCode::Char('L') => Some(Message::ShowLog(LogType::LocalBranches)),
        KeyCode::Char('b') => Some(Message::ShowLog(LogType::AllBranches)),
        KeyCode::Char('a') => Some(Message::ShowLog(LogType::AllReferences)),
        _ => None,
    }
}
