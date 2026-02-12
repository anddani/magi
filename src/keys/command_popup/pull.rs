use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::{
        arguments::{Argument::Pull, PullArgument},
        popup::PullPopupState,
    },
    msg::Message,
};

pub fn keys(key: KeyEvent, arg_mode: bool, state: &PullPopupState) -> Option<Message> {
    if arg_mode {
        return match key.code {
            KeyCode::Char(c) => PullArgument::from_key(c)
                .map(|arg| Message::ToggleArgument(Pull(arg)))
                .or(Some(Message::ExitArgMode)),
            _ => Some(Message::ExitArgMode),
        };
    }

    match key.code {
        KeyCode::Char('u') => {
            if state.upstream.is_some() {
                Some(Message::PullUpstream)
            } else {
                Some(Message::ShowPullUpstreamSelect)
            }
        }
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}
