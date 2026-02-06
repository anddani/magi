use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    model::{arguments::PushArgument, popup::PushPopupState},
    msg::Message,
};

pub fn keys(key: KeyEvent, arg_mode: bool, state: &PushPopupState) -> Option<Message> {
    if state.input_mode {
        // In input mode, handle text input
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('g')) => {
                Some(Message::DismissPopup)
            }
            (KeyModifiers::NONE, KeyCode::Enter) => Some(Message::PushConfirmInput),
            (KeyModifiers::NONE, KeyCode::Backspace) => Some(Message::PushInputBackspace),
            (KeyModifiers::NONE, KeyCode::Tab) => Some(Message::PushInputComplete),
            (KeyModifiers::NONE, KeyCode::Char(c)) | (KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                Some(Message::PushInputChar(c))
            }
            _ => None,
        }
    } else if arg_mode {
        // In argument selection mode
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('g')) => {
                Some(Message::DismissPopup)
            }
            (_, KeyCode::Char('f')) => Some(Message::ToggleArgument(PushArgument::ForceWithLease)),
            (_, KeyCode::Char('F')) => Some(Message::ToggleArgument(PushArgument::Force)),
            // Any other key exits argument mode
            _ => Some(Message::ExitArgMode),
        }
    } else {
        // Normal push popup mode
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('g')) => Some(Message::DismissPopup),
            (KeyModifiers::NONE, KeyCode::Char('u')) => {
                if state.upstream.is_some() {
                    Some(Message::PushUpstream)
                } else {
                    Some(Message::PushEnterInputMode)
                }
            }
            (KeyModifiers::NONE, KeyCode::Char('-')) => Some(Message::EnterArgMode),
            _ => None,
        }
    }
}
