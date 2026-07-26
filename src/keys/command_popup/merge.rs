use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::{
        arguments::{Argument::Merge, MergeArgument},
        popup::MergePopupState,
    },
    msg::{MergeCommand, Message, OnSelect, OptionsSource, ShowSelectPopupConfig},
};

pub fn keys(key: KeyEvent, arg_mode: bool, state: &MergePopupState) -> Option<Message> {
    if state.in_progress {
        return match key.code {
            KeyCode::Char('q') => Some(Message::DismissPopup),
            KeyCode::Char('m') => Some(Message::Merge(MergeCommand::Continue)),
            KeyCode::Char('a') => Some(Message::Merge(MergeCommand::Abort)),
            _ => None,
        };
    }

    if arg_mode {
        return match key.code {
            KeyCode::Char('s') => Some(Message::ShowMergeStrategySelect),
            KeyCode::Char(c) => MergeArgument::from_key(c)
                .map(|arg| Message::ToggleArgument(Merge(arg)))
                .or(Some(Message::ExitArgMode)),
            _ => Some(Message::ExitArgMode),
        };
    }

    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        KeyCode::Char('m') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Merge branch".to_string(),
            source: OptionsSource::LocalAndRemoteBranches,
            on_select: OnSelect::MergeElsewhere,
        })),
        KeyCode::Char('e') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Merge branch (edit message)".to_string(),
            source: OptionsSource::LocalAndRemoteBranches,
            on_select: OnSelect::MergeEditMessage,
        })),
        KeyCode::Char('n') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Merge branch (no commit)".to_string(),
            source: OptionsSource::LocalAndRemoteBranches,
            on_select: OnSelect::MergeNoCommit,
        })),
        KeyCode::Char('a') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Absorb branch".to_string(),
            source: OptionsSource::LocalBranches,
            on_select: OnSelect::MergeAbsorb,
        })),
        KeyCode::Char('p') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Preview merge".to_string(),
            source: OptionsSource::LocalAndRemoteBranches,
            on_select: OnSelect::MergePreview,
        })),
        KeyCode::Char('s') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Squash merge".to_string(),
            source: OptionsSource::LocalAndRemoteBranches,
            on_select: OnSelect::MergeSquash,
        })),
        KeyCode::Char('d') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Dissolve into".to_string(),
            source: OptionsSource::LocalBranches,
            on_select: OnSelect::MergeDissolve,
        })),
        _ => None,
    }
}
