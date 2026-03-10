use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::{
        arguments::{Argument::Fetch, FetchArgument},
        popup::FetchPopupState,
    },
    msg::{FetchCommand, Message, OnSelect, OptionsSource, ShowSelectPopupConfig},
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
                Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                    title: "Fetch from".to_string(),
                    source: OptionsSource::UpstreamBranches,
                    on_select: OnSelect::FetchUpstream,
                }))
            }
        }
        KeyCode::Char('p') => {
            if let Some(remote) = state.push_remote.as_ref().or(state.sole_remote.as_ref()) {
                Some(Message::Fetch(FetchCommand::FetchFromPushRemote(
                    remote.clone(),
                )))
            } else {
                Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                    title: "Fetch from push remote".to_string(),
                    source: OptionsSource::Remotes,
                    on_select: OnSelect::FetchPushRemote,
                }))
            }
        }
        KeyCode::Char('a') => Some(Message::Fetch(FetchCommand::FetchAllRemotes)),
        KeyCode::Char('e') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Fetch from".to_string(),
            source: OptionsSource::Remotes,
            on_select: OnSelect::FetchElsewhere,
        })),
        KeyCode::Char('o') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Fetch branch from".to_string(),
            source: OptionsSource::Remotes,
            on_select: OnSelect::FetchAnotherBranchRemote,
        })),
        KeyCode::Char('m') => Some(Message::Fetch(FetchCommand::FetchModules)),
        KeyCode::Char('r') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Fetch from remote".to_string(),
            source: OptionsSource::Remotes,
            on_select: OnSelect::FetchRefspecRemotePick,
        })),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}
