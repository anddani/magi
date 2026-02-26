use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::{
        arguments::{Argument::Push, PushArgument},
        popup::PushPopupState,
    },
    msg::{Message, PushCommand, SelectDialog},
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
            if let Some(remote) = &state.push_remote {
                Some(Message::Push(PushCommand::PushToPushRemote(remote.clone())))
            } else {
                Some(Message::ShowSelectDialog(SelectDialog::PushPushRemote))
            }
        }
        KeyCode::Char('u') => {
            if state.upstream.is_some() {
                Some(Message::Push(PushCommand::PushUpstream))
            } else {
                Some(Message::ShowSelectDialog(SelectDialog::PushUpstream))
            }
        }
        KeyCode::Char('t') => Some(Message::ShowSelectDialog(SelectDialog::PushAllTags)),
        KeyCode::Char('T') => Some(Message::ShowSelectDialog(SelectDialog::PushTag)),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}
