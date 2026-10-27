use crossterm::event::{KeyCode, KeyEvent};

use crate::i18n;
use crate::msg::{Message, OnSelect, OptionsSource, ResetMode, ShowSelectPopupConfig};

pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('b') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: i18n::t().title_reset_select_branch.to_string(),
            source: OptionsSource::LocalBranches,
            on_select: OnSelect::ResetBranchPick,
        })),
        KeyCode::Char('f') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: i18n::t().title_checkout_file_from_revision.to_string(),
            source: OptionsSource::FileCheckoutRevisions,
            on_select: OnSelect::FileCheckoutRevision,
        })),
        KeyCode::Char('m') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: i18n::t().title_mixed_reset_to.to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::Reset(ResetMode::Mixed),
        })),
        KeyCode::Char('s') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: i18n::t().title_soft_reset_to.to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::Reset(ResetMode::Soft),
        })),
        KeyCode::Char('h') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: i18n::t().title_hard_reset_to.to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::Reset(ResetMode::Hard),
        })),
        KeyCode::Char('i') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: i18n::t().title_reset_index_to.to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::ResetIndex,
        })),
        KeyCode::Char('w') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: i18n::t().title_reset_worktree_to.to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::ResetWorktree,
        })),
        _ => None,
    }
}
