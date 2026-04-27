use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::popup::ApplyPopupState,
    msg::{ApplyCommand, Message, OnSelect, OptionsSource, ShowSelectPopupConfig},
};

pub fn keys(key: KeyEvent, state: &ApplyPopupState) -> Option<Message> {
    if state.in_progress {
        return match key.code {
            KeyCode::Char('q') => Some(Message::DismissPopup),
            KeyCode::Char('A') => Some(Message::Apply(ApplyCommand::Continue)),
            KeyCode::Char('s') => Some(Message::Apply(ApplyCommand::Skip)),
            KeyCode::Char('a') => Some(Message::Apply(ApplyCommand::Abort)),
            _ => None,
        };
    }

    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('A') => {
            if state.selected_commits.is_empty() {
                Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                    title: "Apply (cherry-pick)".to_string(),
                    source: OptionsSource::AllRefs,
                    on_select: OnSelect::ApplyPick,
                }))
            } else {
                Some(Message::Apply(ApplyCommand::Pick(
                    state.selected_commits.clone(),
                )))
            }
        }
        KeyCode::Char('a') => {
            if state.selected_commits.is_empty() {
                Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                    title: "Apply without committing".to_string(),
                    source: OptionsSource::AllRefs,
                    on_select: OnSelect::ApplyApply,
                }))
            } else {
                Some(Message::Apply(ApplyCommand::Apply(
                    state.selected_commits.clone(),
                )))
            }
        }
        KeyCode::Char('m') => {
            if state.selected_commits.is_empty() {
                Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                    title: "Squash commit".to_string(),
                    source: OptionsSource::AllRefs,
                    on_select: OnSelect::ApplySquash,
                }))
            } else {
                Some(Message::Apply(ApplyCommand::Squash(
                    state.selected_commits[0].clone(),
                )))
            }
        }
        KeyCode::Char('n') => {
            if state.selected_commits.is_empty() {
                Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                    title: "Spinout commit".to_string(),
                    source: OptionsSource::AllRefs,
                    on_select: OnSelect::CherrySpinoutCommitPick,
                }))
            } else {
                Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                    title: "Spinout root".to_string(),
                    source: OptionsSource::AllRefs,
                    on_select: OnSelect::CherrySpinoutRootPick {
                        commits: state.selected_commits.clone(),
                    },
                }))
            }
        }
        KeyCode::Char('d') => {
            if state.selected_commits.is_empty() {
                Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                    title: "Donate commit".to_string(),
                    source: OptionsSource::AllRefs,
                    on_select: OnSelect::DonateCommitPick,
                }))
            } else {
                Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                    title: "Donate to branch".to_string(),
                    source: OptionsSource::LocalBranches,
                    on_select: OnSelect::DonateTargetBranch {
                        commits: state.selected_commits.clone(),
                    },
                }))
            }
        }
        KeyCode::Char('h') => {
            if state.selected_commits.is_empty() {
                Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                    title: "Harvest commit".to_string(),
                    source: OptionsSource::AllRefs,
                    on_select: OnSelect::HarvestCommitPick,
                }))
            } else {
                Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                    title: "Harvest from branch".to_string(),
                    source: OptionsSource::LocalBranches,
                    on_select: OnSelect::HarvestSourceBranch {
                        commits: state.selected_commits.clone(),
                    },
                }))
            }
        }
        _ => None,
    }
}
