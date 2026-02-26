use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::{
        arguments::{Argument::Pull, PullArgument},
        popup::PullPopupState,
    },
    msg::{Message, PullCommand, SelectPopup},
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
        KeyCode::Char('p') => {
            if let Some(remote) = &state.push_remote {
                Some(Message::Pull(PullCommand::PullFromPushRemote(
                    remote.clone(),
                )))
            } else {
                Some(Message::ShowSelectPopup(SelectPopup::PullPushRemote))
            }
        }
        KeyCode::Char('u') => {
            if state.upstream.is_some() {
                Some(Message::Pull(PullCommand::PullUpstream))
            } else {
                Some(Message::ShowSelectPopup(SelectPopup::PullUpstream))
            }
        }
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}
