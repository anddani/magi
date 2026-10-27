use crossterm::event::{KeyCode, KeyEvent};

use crate::i18n;
use crate::msg::{Message, OnSelect, OptionsSource, ShowSelectPopupConfig};

pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('b') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: i18n::t().title_worktree_checkout.to_string(),
            source: OptionsSource::BranchesAndTagsExcludingCheckedOut,
            on_select: OnSelect::WorktreeAdd { checkout: true },
        })),
        KeyCode::Char('c') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: i18n::t().title_create_branch_starting_at.to_string(),
            source: OptionsSource::BranchesAndTags,
            on_select: OnSelect::WorktreeBranch,
        })),
        KeyCode::Char('m') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: i18n::t().title_move_worktree.to_string(),
            source: OptionsSource::LinkedWorktrees,
            on_select: OnSelect::WorktreeMove,
        })),
        KeyCode::Char('k') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: i18n::t().title_delete_worktree.to_string(),
            source: OptionsSource::LinkedWorktrees,
            on_select: OnSelect::WorktreeDelete,
        })),
        KeyCode::Char('g') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: i18n::t().title_visit_worktree.to_string(),
            source: OptionsSource::OtherWorktrees,
            on_select: OnSelect::WorktreeVisit,
        })),
        _ => None,
    }
}
