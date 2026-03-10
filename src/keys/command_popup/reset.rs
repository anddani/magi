use crossterm::event::{KeyCode, KeyEvent};

use crate::msg::{Message, OnSelect, OptionsSource, ResetMode, ShowSelectPopupConfig};

pub fn keys(key: KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('b') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Reset: select branch".to_string(),
            source: OptionsSource::LocalBranches,
            on_select: OnSelect::ResetBranchPick,
        })),
        KeyCode::Char('f') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Checkout file from revision".to_string(),
            source: OptionsSource::FileCheckoutRevisions,
            on_select: OnSelect::FileCheckoutRevision,
        })),
        KeyCode::Char('m') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: format!("{} reset to", ResetMode::Mixed.name()),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::Reset(ResetMode::Mixed),
        })),
        KeyCode::Char('s') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: format!("{} reset to", ResetMode::Soft.name()),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::Reset(ResetMode::Soft),
        })),
        KeyCode::Char('h') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: format!("{} reset to", ResetMode::Hard.name()),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::Reset(ResetMode::Hard),
        })),
        KeyCode::Char('i') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Reset index to".to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::ResetIndex,
        })),
        KeyCode::Char('w') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Reset worktree to".to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::ResetWorktree,
        })),
        _ => None,
    }
}
