use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::arguments::{Argument::Log, LogArgument},
    msg::{LogType, Message, OnSelect, OptionsSource, ShowSelectPopupConfig},
};

/// Handle key events for the Log command popup
pub fn keys(key: KeyEvent, arg_mode: bool) -> Option<Message> {
    if arg_mode {
        return match key.code {
            KeyCode::Char(c) => LogArgument::from_key(c)
                .map(|arg| Message::ToggleArgument(Log(arg)))
                .or(Some(Message::ExitArgMode)),
            _ => Some(Message::ExitArgMode),
        };
    }

    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('l') => Some(Message::ShowLog(LogType::Current)),
        KeyCode::Char('o') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Log revision".to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::LogOther,
        })),
        KeyCode::Char('u') => Some(Message::ShowLog(LogType::Related)),
        KeyCode::Char('L') => Some(Message::ShowLog(LogType::LocalBranches)),
        KeyCode::Char('b') => Some(Message::ShowLog(LogType::AllBranches)),
        KeyCode::Char('a') => Some(Message::ShowLog(LogType::AllReferences)),
        KeyCode::Char('r') => Some(Message::ShowLog(LogType::Reflog)),
        KeyCode::Char('O') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Show reflog for".to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::ReflogOther,
        })),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}
