use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::{model::Model, msg::Message};

/// Maps a key event into a [`Message`] given the application state.
/// If function returns [`None`], no action should be triggered.
pub fn handle_key(key: event::KeyEvent, model: &Model) -> Option<Message> {
    // If a dialog is showing, only allow dismissing it
    if model.dialog.is_some() {
        return match key.code {
            KeyCode::Enter | KeyCode::Esc => Some(Message::DismissDialog),
            _ => None,
        };
    }

    // Check for visual mode exit keys first (ESC and Ctrl-g)
    if model.ui_model.is_visual_mode() {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc)
            | (KeyModifiers::CONTROL, KeyCode::Char('g'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                return Some(Message::ExitVisualMode);
            }
            // Disable ToggleSection in visual mode to prevent confusing selection behavior
            (KeyModifiers::NONE, KeyCode::Tab) => {
                return None;
            }
            _ => {}
        }
    }

    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('r')) => Some(Message::Refresh),
        (KeyModifiers::CONTROL, KeyCode::Char('u')) => Some(Message::HalfPageUp),
        (KeyModifiers::CONTROL, KeyCode::Char('d')) => Some(Message::HalfPageDown),
        (KeyModifiers::CONTROL, KeyCode::Char('e')) => Some(Message::ScrollLineDown),
        (KeyModifiers::CONTROL, KeyCode::Char('y')) => Some(Message::ScrollLineUp),
        (KeyModifiers::SHIFT, KeyCode::Char('S')) => Some(Message::StageAllModified),
        (KeyModifiers::SHIFT, KeyCode::Char('U')) => Some(Message::UnstageAll),
        (KeyModifiers::SHIFT, KeyCode::Char('V')) => Some(Message::EnterVisualMode),
        (KeyModifiers::NONE, KeyCode::Char('q')) => Some(Message::Quit),
        (KeyModifiers::NONE, KeyCode::Char('k') | KeyCode::Up) => Some(Message::MoveUp),
        (KeyModifiers::NONE, KeyCode::Char('j') | KeyCode::Down) => Some(Message::MoveDown),
        (KeyModifiers::NONE, KeyCode::Tab) => Some(Message::ToggleSection),
        (KeyModifiers::NONE, KeyCode::Char('c')) => Some(Message::Commit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Theme;
    use crate::git::test_repo::TestRepo;
    use crate::git::GitInfo;
    use crate::model::{RunningState, UiModel};
    use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState};

    fn create_key_event(modifiers: KeyModifiers, code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn create_test_model() -> Model {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();
        let git_info = GitInfo::new_from_path(repo_path).unwrap();
        Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel::default(),
            theme: Theme::default(),
            dialog: None,
            toast: None,
        }
    }

    #[test]
    fn test_shift_v_enters_visual_mode() {
        let model = create_test_model();
        let key = create_key_event(KeyModifiers::SHIFT, KeyCode::Char('V'));

        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::EnterVisualMode));
    }

    #[test]
    fn test_esc_exits_visual_mode() {
        let mut model = create_test_model();
        model.ui_model.visual_mode_anchor = Some(5); // Enter visual mode

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ExitVisualMode));
    }

    #[test]
    fn test_ctrl_g_exits_visual_mode() {
        let mut model = create_test_model();
        model.ui_model.visual_mode_anchor = Some(5); // Enter visual mode

        let key = create_key_event(KeyModifiers::CONTROL, KeyCode::Char('g'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ExitVisualMode));
    }

    #[test]
    fn test_esc_does_nothing_when_not_in_visual_mode() {
        let model = create_test_model();
        let key = create_key_event(KeyModifiers::NONE, KeyCode::Esc);

        let result = handle_key(key, &model);
        assert_eq!(result, None);
    }

    #[test]
    fn test_ctrl_g_does_nothing_when_not_in_visual_mode() {
        let model = create_test_model();
        let key = create_key_event(KeyModifiers::CONTROL, KeyCode::Char('g'));

        let result = handle_key(key, &model);
        assert_eq!(result, None);
    }

    #[test]
    fn test_movement_keys_work_in_visual_mode() {
        let mut model = create_test_model();
        model.ui_model.visual_mode_anchor = Some(5); // Enter visual mode

        // j should still work
        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('j'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::MoveDown));

        // k should still work
        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('k'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::MoveUp));
    }

    #[test]
    fn test_tab_disabled_in_visual_mode() {
        let mut model = create_test_model();
        model.ui_model.visual_mode_anchor = Some(5); // Enter visual mode

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Tab);
        let result = handle_key(key, &model);
        assert_eq!(result, None); // Tab should do nothing in visual mode
    }

    #[test]
    fn test_tab_works_when_not_in_visual_mode() {
        let model = create_test_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Tab);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ToggleSection));
    }
}
