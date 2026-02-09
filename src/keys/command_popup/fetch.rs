use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::{
        arguments::{Argument::Fetch, FetchArgument},
        popup::FetchPopupState,
    },
    msg::Message,
};

pub fn keys(key: KeyEvent, arg_mode: bool, state: &FetchPopupState) -> Option<Message> {
    if arg_mode {
        return match key.code {
            KeyCode::Char('p') => Some(Message::ToggleArgument(Fetch(FetchArgument::Prune))),
            KeyCode::Char('t') => Some(Message::ToggleArgument(Fetch(FetchArgument::Tags))),
            KeyCode::Char('F') => Some(Message::ToggleArgument(Fetch(FetchArgument::Force))),
            // Any other key exits argument mode
            _ => Some(Message::ExitArgMode),
        };
    }

    match key.code {
        KeyCode::Char('u') => {
            if state.upstream.is_some() {
                Some(Message::FetchUpstream)
            } else {
                Some(Message::ShowFetchUpstreamSelect)
            }
        }
        KeyCode::Char('a') => Some(Message::FetchAllRemotes),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}
