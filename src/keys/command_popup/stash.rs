use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    model::arguments::{Argument::Stash, StashArgument},
    msg::{
        LogType, Message, OnSelect, OptionsSource, ShowSelectPopupConfig, StashCommand, StashType,
    },
};

pub fn keys(key: KeyEvent, arg_mode: bool) -> Option<Message> {
    if arg_mode {
        return match key.code {
            KeyCode::Char(c) => StashArgument::from_key(c)
                .map(|arg| Message::ToggleArgument(Stash(arg)))
                .or(Some(Message::ExitArgMode)),
            _ => Some(Message::ExitArgMode),
        };
    }

    match key.code {
        KeyCode::Char('z') => Some(Message::ShowStashInput(StashType::Both)),
        KeyCode::Char('i') => Some(Message::ShowStashInput(StashType::Index)),
        KeyCode::Char('w') => Some(Message::ShowStashInput(StashType::Worktree)),
        KeyCode::Char('x') => Some(Message::ShowStashInput(StashType::KeepingIndex)),
        KeyCode::Char('Z') => Some(Message::Stash(StashCommand::Snapshot)),
        KeyCode::Char('I') => Some(Message::Stash(StashCommand::SnapshotIndex)),
        KeyCode::Char('W') => Some(Message::Stash(StashCommand::SnapshotWorktree)),
        KeyCode::Char('r') => Some(Message::Stash(StashCommand::ToWipRef)),
        KeyCode::Char('a') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Apply stash".to_string(),
            source: OptionsSource::Stashes,
            on_select: OnSelect::ApplyStash,
        })),
        KeyCode::Char('p') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Pop stash".to_string(),
            source: OptionsSource::Stashes,
            on_select: OnSelect::PopStash,
        })),
        KeyCode::Char('k') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Drop stash".to_string(),
            source: OptionsSource::Stashes,
            on_select: OnSelect::DropStash,
        })),
        KeyCode::Char('b') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Branch stash".to_string(),
            source: OptionsSource::Stashes,
            on_select: OnSelect::BranchStash,
        })),
        KeyCode::Char('l') => Some(Message::ShowLog(LogType::Stashes)),
        KeyCode::Char('v') => Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Show stash".to_string(),
            source: OptionsSource::Stashes,
            on_select: OnSelect::ShowStash,
        })),
        KeyCode::Char('-') => Some(Message::EnterArgMode),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn test_v_shows_show_stash_select_popup() {
        let key = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE);
        let result = keys(key, false);
        assert_eq!(
            result,
            Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                title: "Show stash".to_string(),
                source: OptionsSource::Stashes,
                on_select: OnSelect::ShowStash,
            }))
        );
    }

    #[test]
    fn test_b_shows_branch_stash_select_popup() {
        let key = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
        let result = keys(key, false);
        assert_eq!(
            result,
            Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
                title: "Branch stash".to_string(),
                source: OptionsSource::Stashes,
                on_select: OnSelect::BranchStash,
            }))
        );
    }
}
