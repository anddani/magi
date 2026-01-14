use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::{model::Model, msg::Message};

/// Maps a key event into a [`Message`] given the application state.
/// If function returns [`None`], no action should be triggered.
pub fn handle_key(key: event::KeyEvent, model: &Model) -> Option<Message> {
    // If a dialog is showing, only allow dismissing it
    if model.dialog.is_some() {
        return match key.code {
            KeyCode::Enter | KeyCode::Esc => Some(Message::DismissDialog),
            _ => None,
        };
    }

    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => Some(Message::Quit),
        (KeyModifiers::CONTROL, KeyCode::Char('r')) => Some(Message::Refresh),
        (KeyModifiers::CONTROL, KeyCode::Char('u')) => Some(Message::HalfPageUp),
        (KeyModifiers::CONTROL, KeyCode::Char('d')) => Some(Message::HalfPageDown),
        (KeyModifiers::CONTROL, KeyCode::Char('e')) => Some(Message::ScrollLineDown),
        (KeyModifiers::CONTROL, KeyCode::Char('y')) => Some(Message::ScrollLineUp),
        (KeyModifiers::SHIFT, KeyCode::Char('S')) => Some(Message::StageAllModified),
        (KeyModifiers::SHIFT, KeyCode::Char('U')) => Some(Message::UnstageAll),
        (KeyModifiers::NONE, KeyCode::Char('q')) => Some(Message::Quit),
        (KeyModifiers::NONE, KeyCode::Char('k') | KeyCode::Up) => Some(Message::MoveUp),
        (KeyModifiers::NONE, KeyCode::Char('j') | KeyCode::Down) => Some(Message::MoveDown),
        (KeyModifiers::NONE, KeyCode::Tab) => Some(Message::ToggleSection),
        (KeyModifiers::NONE, KeyCode::Char('c')) => Some(Message::Commit),
        _ => None,
    }
}
