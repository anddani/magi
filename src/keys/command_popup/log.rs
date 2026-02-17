use crossterm::event::{KeyCode, KeyEvent};

use crate::msg::{LogType, Message};

/// Handle key events for the Log command popup
pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('l') => Some(Message::ShowLog(LogType::Current)),
        KeyCode::Char('L') => Some(Message::ShowLog(LogType::LocalBranches)),
        KeyCode::Char('a') => Some(Message::ShowLog(LogType::AllReferences)),
        _ => None,
    }
}
