use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::{
    model::{
        popup::{PopupContent, PopupContentCommand},
        Model,
    },
    msg::Message,
};

/// Maps a key event into a [`Message`] given the application state.
/// If function returns [`None`], no action should be triggered.
pub fn handle_key(key: event::KeyEvent, model: &Model) -> Option<Message> {
    if let Some(PopupContent::Error { .. }) = &model.popup {
        return match key.code {
            KeyCode::Enter | KeyCode::Esc => Some(Message::DismissPopup),
            _ => None,
        };
    }

    if let Some(PopupContent::Command(command)) = &model.popup {
        return match command {
            PopupContentCommand::Help => match (key.modifiers, key.code) {
                (KeyModifiers::NONE, KeyCode::Esc | KeyCode::Char('q'))
                | (KeyModifiers::CONTROL, KeyCode::Char('g')) => Some(Message::DismissPopup),
                _ => None,
            },
            PopupContentCommand::Commit => match (key.modifiers, key.code) {
                (KeyModifiers::NONE, KeyCode::Esc | KeyCode::Char('q'))
                | (KeyModifiers::CONTROL, KeyCode::Char('g')) => Some(Message::DismissPopup),
                (KeyModifiers::NONE, KeyCode::Char('c')) => Some(Message::Commit),
                (KeyModifiers::NONE, KeyCode::Char('a')) => Some(Message::Amend),
                _ => None,
            },
            PopupContentCommand::Push(state) => {
                if state.input_mode {
                    // In input mode, handle text input
                    match (key.modifiers, key.code) {
                        (KeyModifiers::NONE, KeyCode::Esc)
                        | (KeyModifiers::CONTROL, KeyCode::Char('g')) => {
                            Some(Message::DismissPopup)
                        }
                        (KeyModifiers::NONE, KeyCode::Enter) => Some(Message::PushConfirmInput),
                        (KeyModifiers::NONE, KeyCode::Backspace) => {
                            Some(Message::PushInputBackspace)
                        }
                        (KeyModifiers::NONE, KeyCode::Char(c))
                        | (KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                            Some(Message::PushInputChar(c))
                        }
                        _ => None,
                    }
                } else {
                    // Normal push popup mode
                    match (key.modifiers, key.code) {
                        (KeyModifiers::NONE, KeyCode::Esc | KeyCode::Char('q'))
                        | (KeyModifiers::CONTROL, KeyCode::Char('g')) => {
                            Some(Message::DismissPopup)
                        }
                        (KeyModifiers::NONE, KeyCode::Char('u')) => {
                            if state.upstream.is_some() {
                                Some(Message::PushUpstream)
                            } else {
                                Some(Message::PushEnterInputMode)
                            }
                        }
                        _ => None,
                    }
                }
            }
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
        (KeyModifiers::SHIFT, KeyCode::Char('P')) => Some(Message::ShowPushPopup),
        (KeyModifiers::NONE, KeyCode::Char('?')) => Some(Message::ShowHelp),
        (KeyModifiers::NONE, KeyCode::Char('q')) => Some(Message::Quit),
        (KeyModifiers::NONE, KeyCode::Char('k') | KeyCode::Up) => Some(Message::MoveUp),
        (KeyModifiers::NONE, KeyCode::Char('j') | KeyCode::Down) => Some(Message::MoveDown),
        (KeyModifiers::NONE, KeyCode::Tab) => Some(Message::ToggleSection),
        (KeyModifiers::NONE, KeyCode::Char('c')) => Some(Message::ShowCommitPopup),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Theme;
    use crate::git::test_repo::TestRepo;
    use crate::git::GitInfo;
    use crate::model::popup::PopupContentCommand;
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
            popup: None,
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

    #[test]
    fn test_question_mark_shows_help() {
        let model = create_test_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('?'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowHelp));
    }

    #[test]
    fn test_question_mark_does_nothing_when_error_popup_shown() {
        let mut model = create_test_model();
        model.popup = Some(PopupContent::Error {
            message: "Test error".to_string(),
        });

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('?'));
        let result = handle_key(key, &model);
        // When error popup is shown, only Enter/Esc should work
        assert_eq!(result, None);
    }

    #[test]
    fn test_esc_dismisses_help_popup() {
        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Help));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_q_dismisses_help_popup() {
        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Help));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('q'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_c_shows_commit_popup() {
        let model = create_test_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('c'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowCommitPopup));
    }

    #[test]
    fn test_c_in_commit_popup_triggers_commit() {
        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Commit));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('c'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Commit));
    }

    #[test]
    fn test_esc_dismisses_commit_popup() {
        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Commit));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_q_dismisses_commit_popup() {
        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Commit));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('q'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_a_in_commit_popup_triggers_amend() {
        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Commit));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('a'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Amend));
    }

    #[test]
    fn test_shift_p_shows_push_popup() {
        let model = create_test_model();

        let key = create_key_event(KeyModifiers::SHIFT, KeyCode::Char('P'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowPushPopup));
    }

    #[test]
    fn test_u_in_push_popup_with_upstream_pushes() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                local_branch: "main".to_string(),
                upstream: Some("origin/main".to_string()),
                input_mode: false,
                input_text: String::new(),
            },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('u'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::PushUpstream));
    }

    #[test]
    fn test_u_in_push_popup_without_upstream_enters_input_mode() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                local_branch: "main".to_string(),
                upstream: None,
                input_mode: false,
                input_text: String::new(),
            },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('u'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::PushEnterInputMode));
    }

    #[test]
    fn test_esc_dismisses_push_popup() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                local_branch: "main".to_string(),
                upstream: None,
                input_mode: false,
                input_text: String::new(),
            },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_push_popup_input_mode_handles_chars() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                local_branch: "main".to_string(),
                upstream: None,
                input_mode: true,
                input_text: String::new(),
            },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('a'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::PushInputChar('a')));
    }

    #[test]
    fn test_push_popup_input_mode_handles_backspace() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                local_branch: "main".to_string(),
                upstream: None,
                input_mode: true,
                input_text: "test".to_string(),
            },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Backspace);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::PushInputBackspace));
    }

    #[test]
    fn test_push_popup_input_mode_handles_enter() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                local_branch: "main".to_string(),
                upstream: None,
                input_mode: true,
                input_text: "feature".to_string(),
            },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Enter);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::PushConfirmInput));
    }

    #[test]
    fn test_push_popup_input_mode_esc_dismisses() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                local_branch: "main".to_string(),
                upstream: None,
                input_mode: true,
                input_text: String::new(),
            },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }
}
