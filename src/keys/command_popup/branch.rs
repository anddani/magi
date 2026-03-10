use crossterm::event::{KeyCode, KeyEvent};

use crate::msg::{Message, OnSelect, OptionsSource, ShowSelectPopupConfig};

pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('b') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Checkout".to_string(),
            source: OptionsSource::LocalAndRemoteBranches,
            on_select: OnSelect::CheckoutBranch,
        })),
        KeyCode::Char('l') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Checkout local".to_string(),
            source: OptionsSource::LocalBranches,
            on_select: OnSelect::CheckoutLocalBranch,
        })),
        KeyCode::Char('c') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Create branch starting at".to_string(),
            source: OptionsSource::BranchesAndTags,
            on_select: OnSelect::CreateNewBranchBase { checkout: true },
        })),
        KeyCode::Char('n') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Create branch starting at".to_string(),
            source: OptionsSource::BranchesAndTags,
            on_select: OnSelect::CreateNewBranchBase { checkout: false },
        })),
        KeyCode::Char('s') => Some(Message::ShowSpinoffBranchInput),
        KeyCode::Char('S') => Some(Message::ShowSpinoutBranchInput),
        KeyCode::Char('w') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Worktree checkout".to_string(),
            source: OptionsSource::BranchesAndTagsExcludingCheckedOut,
            on_select: OnSelect::WorktreeAdd { checkout: true },
        })),
        KeyCode::Char('W') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Worktree create".to_string(),
            source: OptionsSource::BranchesAndTagsExcludingCheckedOut,
            on_select: OnSelect::WorktreeAdd { checkout: false },
        })),
        KeyCode::Char('m') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Rename branch".to_string(),
            source: OptionsSource::LocalBranches,
            on_select: OnSelect::RenameBranch,
        })),
        KeyCode::Char('x') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Delete branch".to_string(),
            source: OptionsSource::LocalAndRemoteBranches,
            on_select: OnSelect::DeleteBranch,
        })),
        KeyCode::Char('X') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Reset: select branch".to_string(),
            source: OptionsSource::LocalBranches,
            on_select: OnSelect::ResetBranchPick,
        })),
        KeyCode::Char('o') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Open PR".to_string(),
            source: OptionsSource::LocalBranchesWithRemote,
            on_select: OnSelect::OpenPrBranch,
        })),
        KeyCode::Char('O') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Open PR".to_string(),
            source: OptionsSource::LocalBranchesWithRemote,
            on_select: OnSelect::OpenPrBranchWithTarget,
        })),
        _ => None,
    }
}
