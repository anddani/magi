use crossterm::event::{KeyCode, KeyEvent};

use crate::msg::{Message, OnSelect, OptionsSource, ShowSelectPopupConfig};

pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('b') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Worktree checkout".to_string(),
            source: OptionsSource::BranchesAndTagsExcludingCheckedOut,
            on_select: OnSelect::WorktreeAdd { checkout: true },
        })),
        KeyCode::Char('c') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Create branch starting at".to_string(),
            source: OptionsSource::BranchesAndTags,
            on_select: OnSelect::WorktreeBranch,
        })),
        _ => None,
    }
}
