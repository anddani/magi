use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::{
        arguments::{Argument::Push, PushArgument},
        popup::PushPopupState,
    },
    msg::Message,
};

pub fn keys(key: KeyEvent, arg_mode: bool, state: &PushPopupState) -> Option<Message> {
    if state.input_mode {
        return match key.code {
            KeyCode::Enter => Some(Message::PushConfirmInput),
            KeyCode::Backspace => Some(Message::PushInputBackspace),
            KeyCode::Tab => Some(Message::PushInputComplete),
            KeyCode::Char(c) => Some(Message::PushInputChar(c)),
            _ => None,
        };
    }

    if arg_mode {
        return match key.code {
            KeyCode::Char('f') => Some(Message::ToggleArgument(Push(PushArgument::ForceWithLease))),
            KeyCode::Char('F') => Some(Message::ToggleArgument(Push(PushArgument::Force))),
            // Any other key exits argument mode
            _ => Some(Message::ExitArgMode),
        };
    }

    match key.code {
        KeyCode::Char('u') => {
            if state.upstream.is_some() {
                Some(Message::PushUpstream)
            } else {
                Some(Message::PushEnterInputMode)
            }
        }
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}
