use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::{
        arguments::{Argument::Push, PushArgument},
        popup::PushPopupState,
    },
    msg::{Message, PushCommand, SelectPopup},
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
        KeyCode::Char('p') => {
            if let Some(remote) = state.push_remote.as_ref().or(state.sole_remote.as_ref()) {
                Some(Message::Push(PushCommand::PushToPushRemote(remote.clone())))
            } else {
                Some(Message::ShowSelectPopup(SelectPopup::PushPushRemote))
            }
        }
        KeyCode::Char('u') => {
            if state.upstream.is_some() {
                Some(Message::Push(PushCommand::PushUpstream))
            } else {
                Some(Message::ShowSelectPopup(SelectPopup::PushUpstream))
            }
        }
        KeyCode::Char('t') => Some(Message::ShowSelectPopup(SelectPopup::PushAllTags)),
        KeyCode::Char('T') => Some(Message::ShowSelectPopup(SelectPopup::PushTag)),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}
