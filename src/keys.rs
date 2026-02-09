use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::{
    model::{popup::PopupContent, Model},
    msg::Message,
};

mod command_popup;
mod credentials_popup;

fn command_popup_keys(c: char) -> Option<Message> {
    match c {
        'S' => Some(Message::StageAllModified),
        'U' => Some(Message::UnstageAll),
        'V' => Some(Message::EnterVisualMode),
        'p' => Some(Message::ShowPushPopup),
        'P' => Some(Message::ShowPushPopup),
        'c' => Some(Message::ShowCommitPopup),
        'b' => Some(Message::ShowBranchPopup),
        _ => None,
    }
}

/// Maps a key event into a [`Message`] given the application state.
/// If function returns [`None`], no action should be triggered.
pub fn handle_key(key: event::KeyEvent, model: &Model) -> Option<Message> {
    if let Some(PopupContent::Error { .. }) = &model.popup {
        return match key.code {
            KeyCode::Enter | KeyCode::Esc => Some(Message::DismissPopup),
            _ => None,
        };
    }

    if let Some(PopupContent::Credential(_)) = &model.popup {
        return credentials_popup::handle_credentials_popup_key(key);
    }

    if let Some(PopupContent::Command(command)) = &model.popup {
        return command_popup::handle_command_popup_key(key, command, model.arg_mode);
    }

    // Let commands from help popup open dialogs
    if model.popup == Some(PopupContent::Help) {
        return match (key.modifiers, key.code) {
            (_, KeyCode::Esc)
            | (_, KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('g')) => Some(Message::DismissPopup),
            (_, KeyCode::Char(c)) => command_popup_keys(c),
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
        (_, KeyCode::Char('?')) => Some(Message::ShowHelp),
        (_, KeyCode::Char('q')) => Some(Message::Quit),

        // Navigation
        (KeyModifiers::CONTROL, KeyCode::Char('u')) => Some(Message::HalfPageUp),
        (KeyModifiers::CONTROL, KeyCode::Char('d')) => Some(Message::HalfPageDown),
        (KeyModifiers::CONTROL, KeyCode::Char('e')) => Some(Message::ScrollLineDown),
        (KeyModifiers::CONTROL, KeyCode::Char('y')) => Some(Message::ScrollLineUp),
        (_, KeyCode::Char('k') | KeyCode::Up) => Some(Message::MoveUp),
        (_, KeyCode::Char('j') | KeyCode::Down) => Some(Message::MoveDown),
        (_, KeyCode::Tab) => Some(Message::ToggleSection),

        (_, KeyCode::Char(c)) => command_popup_keys(c),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Theme;
    use crate::git::test_repo::TestRepo;
    use crate::git::GitInfo;
    use crate::model::arguments::PushArgument;
    use crate::model::popup::PopupContentCommand;
    use crate::model::{RunningState, UiModel};
    use crate::msg::SelectMessage;
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
            select_result: None,
            select_context: None,
            pty_state: None,
            arg_mode: false,
            arguments: None,
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
        model.popup = Some(PopupContent::Help);

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_q_dismisses_help_popup() {
        let mut model = create_test_model();
        model.popup = Some(PopupContent::Help);

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
                default_remote: "origin".to_string(),
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
                default_remote: "origin".to_string(),
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
                default_remote: "origin".to_string(),
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
                default_remote: "origin".to_string(),
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
                default_remote: "origin".to_string(),
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
                default_remote: "origin".to_string(),
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
                default_remote: "origin".to_string(),
                input_mode: true,
                input_text: String::new(),
            },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_minus_in_push_popup_enters_arg_mode() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                local_branch: "main".to_string(),
                upstream: None,
                default_remote: "origin".to_string(),
                input_mode: false,
                input_text: String::new(),
            },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('-'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::EnterArgMode));
    }

    #[test]
    fn test_f_in_arg_mode_toggles_force_with_lease() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.arg_mode = true;
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                local_branch: "main".to_string(),
                upstream: None,
                default_remote: "origin".to_string(),
                input_mode: false,
                input_text: String::new(),
            },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('f'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ToggleArgument(PushArgument::ForceWithLease))
        );
    }

    #[test]
    fn test_other_key_in_arg_mode_exits_arg_mode() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.arg_mode = true;
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                local_branch: "main".to_string(),
                upstream: None,
                default_remote: "origin".to_string(),
                input_mode: false,
                input_text: String::new(),
            },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('x'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ExitArgMode));
    }

    #[test]
    fn test_esc_in_arg_mode_dismisses_popup() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                local_branch: "main".to_string(),
                upstream: None,
                default_remote: "origin".to_string(),
                input_mode: false,
                input_text: String::new(),
            },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    // Select popup tests

    fn create_select_popup_model() -> Model {
        use crate::model::popup::SelectPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
            SelectPopupState::new(
                "Checkout".to_string(),
                vec!["main".to_string(), "feature".to_string()],
            ),
        )));
        model
    }

    #[test]
    fn test_select_popup_esc_dismisses() {
        let model = create_select_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_select_popup_ctrl_g_dismisses() {
        let model = create_select_popup_model();

        let key = create_key_event(KeyModifiers::CONTROL, KeyCode::Char('g'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_select_popup_enter_confirms() {
        let model = create_select_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Enter);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Select(SelectMessage::Confirm)));
    }

    #[test]
    fn test_select_popup_char_input() {
        let model = create_select_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('a'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Select(SelectMessage::InputChar('a'))));
    }

    #[test]
    fn test_select_popup_shift_char_input() {
        let model = create_select_popup_model();

        let key = create_key_event(KeyModifiers::SHIFT, KeyCode::Char('A'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Select(SelectMessage::InputChar('A'))));
    }

    #[test]
    fn test_select_popup_backspace() {
        let model = create_select_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Backspace);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Select(SelectMessage::InputBackspace)));
    }

    #[test]
    fn test_select_popup_up_arrow() {
        let model = create_select_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Up);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Select(SelectMessage::MoveUp)));
    }

    #[test]
    fn test_select_popup_down_arrow() {
        let model = create_select_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Down);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Select(SelectMessage::MoveDown)));
    }

    #[test]
    fn test_select_popup_ctrl_p_moves_up() {
        let model = create_select_popup_model();

        let key = create_key_event(KeyModifiers::CONTROL, KeyCode::Char('p'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Select(SelectMessage::MoveUp)));
    }

    #[test]
    fn test_select_popup_ctrl_n_moves_down() {
        let model = create_select_popup_model();

        let key = create_key_event(KeyModifiers::CONTROL, KeyCode::Char('n'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Select(SelectMessage::MoveDown)));
    }

    // Branch popup tests

    #[test]
    fn test_b_shows_branch_popup() {
        let model = create_test_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('b'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowBranchPopup));
    }

    fn create_branch_popup_model() -> Model {
        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Branch));
        model
    }

    #[test]
    fn test_esc_dismisses_branch_popup() {
        let model = create_branch_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_q_dismisses_branch_popup() {
        let model = create_branch_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('q'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_b_in_branch_popup_shows_checkout_select() {
        let model = create_branch_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('b'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowCheckoutBranchPopup));
    }
}
