use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::{
        arguments::{Argument::Push, PushArgument},
        popup::PushPopupState,
    },
    msg::{Message, OnSelect, OptionsSource, PushCommand, ShowSelectPopupConfig},
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
                Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                    title: "Push to push remote".to_string(),
                    source: OptionsSource::Remotes,
                    on_select: OnSelect::PushPushRemote,
                }))
            }
        }
        KeyCode::Char('u') => {
            if state.upstream.is_some() {
                Some(Message::Push(PushCommand::PushUpstream))
            } else {
                Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                    title: "Push to".to_string(),
                    source: OptionsSource::UpstreamBranches,
                    on_select: OnSelect::PushUpstream,
                }))
            }
        }
        KeyCode::Char('e') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Push to".to_string(),
            source: OptionsSource::UpstreamBranches,
            on_select: OnSelect::PushElsewhere,
        })),
        KeyCode::Char('o') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Push branch".to_string(),
            source: OptionsSource::LocalBranches,
            on_select: OnSelect::PushOtherBranchPick,
        })),
        KeyCode::Char('m') => {
            if let Some(remote) = state.sole_remote.as_ref().or(state.push_remote.as_ref()) {
                Some(Message::Push(PushCommand::PushMatching(remote.clone())))
            } else {
                Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                    title: "Push matching branches to".to_string(),
                    source: OptionsSource::Remotes,
                    on_select: OnSelect::PushMatching,
                }))
            }
        }
        KeyCode::Char('r') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Push to remote".to_string(),
            source: OptionsSource::Remotes,
            on_select: OnSelect::PushRefspecRemotePick,
        })),
        KeyCode::Char('t') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Push tags to".to_string(),
            source: OptionsSource::Remotes,
            on_select: OnSelect::PushAllTags,
        })),
        KeyCode::Char('T') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Push tag".to_string(),
            source: OptionsSource::Tags,
            on_select: OnSelect::PushTag,
        })),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}
