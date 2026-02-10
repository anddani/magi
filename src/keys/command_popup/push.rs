use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::{
        arguments::{Argument::Push, PushArgument},
        popup::PushPopupState,
    },
    msg::Message,
};

pub fn keys(key: KeyEvent, arg_mode: bool, state: &PushPopupState) -> Option<Message> {
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
                Some(Message::ShowPushUpstreamSelect)
            }
        }
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}
