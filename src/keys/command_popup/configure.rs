use crossterm::event::{KeyCode, KeyEvent};

use crate::model::popup::ConfigurePopupState;
use crate::msg::{ConfigAction, Message};

pub fn keys(key: KeyEvent, state: &ConfigurePopupState) -> Option<Message> {
    // When the `a` prefix is pending, the next key picks the branch-creation
    // variable to cycle (`m` = autoSetupMerge, `r` = autoSetupRebase). Any
    // other key cancels the prefix. Mirrors magit's `a m` / `a r`.
    if state.pending_a {
        return match key.code {
            KeyCode::Char('m') => Some(Message::ConfigAction(ConfigAction::CycleAutoSetupMerge)),
            KeyCode::Char('r') => Some(Message::ConfigAction(ConfigAction::CycleAutoSetupRebase)),
            _ => Some(Message::ConfigAction(ConfigAction::ClearPendingA)),
        };
    }

    match key.code {
        KeyCode::Char('q') => Some(Message::DismissPopup),
        KeyCode::Char('d') => Some(Message::ConfigAction(ConfigAction::EditDescription)),
        KeyCode::Char('u') => Some(Message::ConfigAction(ConfigAction::ToggleUpstream)),
        KeyCode::Char('r') => Some(Message::ConfigAction(ConfigAction::CycleBranchRebase)),
        KeyCode::Char('p') => Some(Message::ConfigAction(ConfigAction::CycleBranchPushRemote)),
        KeyCode::Char('R') => Some(Message::ConfigAction(ConfigAction::CyclePullRebase)),
        KeyCode::Char('P') => Some(Message::ConfigAction(ConfigAction::CyclePushDefault)),
        KeyCode::Char('a') => Some(Message::ConfigAction(ConfigAction::SetPendingA)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState, KeyModifiers};

    fn key(c: char) -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn state(pending_a: bool) -> ConfigurePopupState {
        ConfigurePopupState {
            branch: "main".to_string(),
            pending_a,
        }
    }

    #[test]
    fn maps_primary_keys() {
        let s = state(false);
        assert_eq!(
            keys(key('d'), &s),
            Some(Message::ConfigAction(ConfigAction::EditDescription))
        );
        assert_eq!(
            keys(key('u'), &s),
            Some(Message::ConfigAction(ConfigAction::ToggleUpstream))
        );
        assert_eq!(
            keys(key('r'), &s),
            Some(Message::ConfigAction(ConfigAction::CycleBranchRebase))
        );
        assert_eq!(
            keys(key('p'), &s),
            Some(Message::ConfigAction(ConfigAction::CycleBranchPushRemote))
        );
        assert_eq!(
            keys(key('R'), &s),
            Some(Message::ConfigAction(ConfigAction::CyclePullRebase))
        );
        assert_eq!(
            keys(key('P'), &s),
            Some(Message::ConfigAction(ConfigAction::CyclePushDefault))
        );
        assert_eq!(
            keys(key('a'), &s),
            Some(Message::ConfigAction(ConfigAction::SetPendingA))
        );
        assert_eq!(keys(key('q'), &s), Some(Message::DismissPopup));
        assert_eq!(keys(key('z'), &s), None);
    }

    #[test]
    fn pending_a_maps_m_and_r_to_autosetup() {
        let s = state(true);
        assert_eq!(
            keys(key('m'), &s),
            Some(Message::ConfigAction(ConfigAction::CycleAutoSetupMerge))
        );
        assert_eq!(
            keys(key('r'), &s),
            Some(Message::ConfigAction(ConfigAction::CycleAutoSetupRebase))
        );
        // Any other key cancels the `a` prefix.
        assert_eq!(
            keys(key('x'), &s),
            Some(Message::ConfigAction(ConfigAction::ClearPendingA))
        );
    }
}
