use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::{
        arguments::{Argument::Fetch, FetchArgument},
        popup::FetchPopupState,
    },
    msg::{FetchCommand, Message, ShowSelectDialog},
};

pub fn keys(key: KeyEvent, arg_mode: bool, state: &FetchPopupState) -> Option<Message> {
    if arg_mode {
        return match key.code {
            KeyCode::Char(c) => FetchArgument::from_key(c)
                .map(|arg| Message::ToggleArgument(Fetch(arg)))
                .or(Some(Message::ExitArgMode)),
            _ => Some(Message::ExitArgMode),
        };
    }

    match key.code {
        KeyCode::Char('u') => {
            if state.upstream.is_some() {
                Some(Message::Fetch(FetchCommand::FetchUpstream))
            } else {
                Some(Message::ShowSelect(ShowSelectDialog::FetchUpstream))
            }
        }
        KeyCode::Char('p') => {
            if let Some(remote) = &state.push_remote {
                Some(Message::Fetch(FetchCommand::FetchFromPushRemote(
                    remote.clone(),
                )))
            } else {
                Some(Message::ShowSelect(ShowSelectDialog::FetchPushRemote))
            }
        }
        KeyCode::Char('a') => Some(Message::Fetch(FetchCommand::FetchAllRemotes)),
        KeyCode::Char('e') => Some(Message::ShowSelect(ShowSelectDialog::FetchElsewhere)),
        KeyCode::Char('o') => Some(Message::ShowSelect(ShowSelectDialog::FetchAnotherBranch)),
        KeyCode::Char('m') => Some(Message::Fetch(FetchCommand::FetchModules)),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}
