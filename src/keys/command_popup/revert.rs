use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::arguments::{Argument::Revert, RevertArgument},
    model::popup::RevertPopupState,
    msg::{Message, RevertCommand},
};

pub fn keys(
    key: KeyEvent,
    arg_mode: bool,
    equals_arg_mode: bool,
    state: &RevertPopupState,
) -> Option<Message> {
    if arg_mode {
        return match key.code {
            KeyCode::Char('m') => Some(Message::ShowRevertMainlineInput),
            KeyCode::Char(c) => RevertArgument::from_key(c)
                .map(|arg| Message::ToggleArgument(Revert(arg)))
                .or(Some(Message::ExitArgMode)),
            _ => Some(Message::ExitArgMode),
        };
    }
    if equals_arg_mode {
        return match key.code {
            KeyCode::Char('s') => Some(Message::ShowRevertStrategySelect),
            _ => Some(Message::ExitArgMode),
        };
    }
    if state.in_progress {
        return match key.code {
            KeyCode::Char('q') => Some(Message::DismissPopup),
            KeyCode::Char('_') => Some(Message::Revert(RevertCommand::Continue)),
            KeyCode::Char('s') => Some(Message::Revert(RevertCommand::Skip)),
            KeyCode::Char('a') => Some(Message::Revert(RevertCommand::Abort)),
            _ => None,
        };
    }

    let has_commits = !state.selected_commits.is_empty();
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('_') if has_commits => Some(Message::Revert(RevertCommand::Commits {
            hashes: state.selected_commits.clone(),
            mainline: state.mainline.clone(),
            strategy: state.strategy.clone(),
        })),
        KeyCode::Char('v') if has_commits => Some(Message::Revert(RevertCommand::NoCommit {
            hashes: state.selected_commits.clone(),
            mainline: state.mainline.clone(),
            strategy: state.strategy.clone(),
        })),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        KeyCode::Char('=') => Some(Message::EnterEqualsArgMode),
        _ => None,
    }
}
