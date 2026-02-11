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
            KeyCode::Char(c) => PushArgument::from_key(c)
                .map(|arg| Message::ToggleArgument(Push(arg)))
                .or(Some(Message::ExitArgMode)),
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
        KeyCode::Char('t') => Some(Message::ShowPushAllTagsSelect),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}
