use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::{
    model::Model,
    model::ViewMode,
    model::popup::{ConfirmAction, PopupContent, PopupContentCommand},
    msg::Message,
};

mod command_popup;
mod credentials_popup;
mod input_popup;

fn command_popup_keys(c: char) -> Option<Message> {
    match c {
        'p' | 'P' => Some(Message::ShowPushPopup),
        'f' => Some(Message::ShowFetchPopup),
        'F' => Some(Message::ShowPullPopup),
        'c' => Some(Message::ShowPopup(PopupContent::Command(
            PopupContentCommand::Commit,
        ))),
        'b' => Some(Message::ShowPopup(PopupContent::Command(
            PopupContentCommand::Branch,
        ))),
        'l' => Some(Message::ShowPopup(PopupContent::Command(
            PopupContentCommand::Log,
        ))),
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

    if let Some(PopupContent::Confirm(state)) = &model.popup {
        return match (key.modifiers, key.code) {
            (_, KeyCode::Char('y')) | (_, KeyCode::Enter) => {
                let msg = match &state.on_confirm {
                    ConfirmAction::DeleteBranch(branch) => {
                        Message::ConfirmDeleteBranch(branch.clone())
                    }
                };
                Some(msg)
            }
            (_, KeyCode::Char('n'))
            | (_, KeyCode::Esc)
            | (KeyModifiers::CONTROL, KeyCode::Char('c'))
            | (KeyModifiers::CONTROL, KeyCode::Char('g')) => Some(Message::DismissPopup),
            _ => None,
        };
    }

    if let Some(PopupContent::Credential(_)) = &model.popup {
        return credentials_popup::handle_credentials_popup_key(key);
    }

    if let Some(PopupContent::Input(_)) = &model.popup {
        return input_popup::handle_input_popup_key(key);
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

    // Handle pending 'g' for 'gg' (go to first line)
    if model.pending_g {
        if key.modifiers == KeyModifiers::NONE && key.code == KeyCode::Char('g') {
            return Some(Message::MoveToTop);
        }
        if key.modifiers == KeyModifiers::NONE && key.code == KeyCode::Char('r') {
            return Some(Message::Refresh);
        }
        // Any other key cancels the pending 'g' and falls through to normal handling
    }

    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('r')) => Some(Message::Refresh),
        (_, KeyCode::Char('?')) => Some(Message::ShowPopup(PopupContent::Help)),
        // 'q' exits log view when in log mode, otherwise quits the app
        (_, KeyCode::Char('q')) => match model.view_mode {
            ViewMode::Log => Some(Message::ExitLogView),
            ViewMode::Status => Some(Message::Quit),
        },
        (_, KeyCode::Char('V')) => Some(Message::EnterVisualMode),
        (_, KeyCode::Char('S')) => Some(Message::StageAllModified),
        (_, KeyCode::Char('U')) => Some(Message::UnstageAll),

        // Navigation
        (KeyModifiers::CONTROL, KeyCode::Char('u')) => Some(Message::HalfPageUp),
        (KeyModifiers::CONTROL, KeyCode::Char('d')) => Some(Message::HalfPageDown),
        (KeyModifiers::CONTROL, KeyCode::Char('e')) => Some(Message::ScrollLineDown),
        (KeyModifiers::CONTROL, KeyCode::Char('y')) => Some(Message::ScrollLineUp),
        (_, KeyCode::Char('k') | KeyCode::Up) => Some(Message::MoveUp),
        (_, KeyCode::Char('j') | KeyCode::Down) => Some(Message::MoveDown),
        (KeyModifiers::NONE, KeyCode::Char('g')) => Some(Message::PendingG),
        (_, KeyCode::Char('G')) => Some(Message::MoveToBottom),
        (_, KeyCode::Tab) => Some(Message::ToggleSection),

        (_, KeyCode::Char(c)) => command_popup_keys(c),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Theme;
    use crate::git::GitInfo;
    use crate::git::test_repo::TestRepo;
    use crate::model::arguments::Argument::{Fetch, Push};
    use crate::model::arguments::{FetchArgument, PushArgument};
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
        use crate::model::ViewMode;

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
            pending_g: false,
            arguments: None,
            open_pr_branch: None,
            view_mode: ViewMode::Status,
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
        assert_eq!(result, Some(Message::ShowPopup(PopupContent::Help)));
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
        assert_eq!(
            result,
            Some(Message::ShowPopup(PopupContent::Command(
                PopupContentCommand::Commit
            )))
        );
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
                upstream: Some("origin/main".to_string()),
            },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('u'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::PushUpstream));
    }

    #[test]
    fn test_u_in_push_popup_without_upstream_shows_select() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('u'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowPushUpstreamSelect));
    }

    #[test]
    fn test_t_in_push_popup_shows_push_all_tags_select() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('t'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowPushAllTagsSelect));
    }

    #[test]
    fn test_shift_t_in_push_popup_shows_push_tag_select() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::SHIFT, KeyCode::Char('T'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowPushTagSelect));
    }

    #[test]
    fn test_esc_dismisses_push_popup() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState { upstream: None },
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
            PushPopupState { upstream: None },
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
            PushPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('f'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ToggleArgument(Push(PushArgument::ForceWithLease)))
        );
    }

    #[test]
    fn test_other_key_in_arg_mode_exits_arg_mode() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.arg_mode = true;
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState { upstream: None },
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
            PushPopupState { upstream: None },
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
        assert_eq!(
            result,
            Some(Message::ShowPopup(PopupContent::Command(
                PopupContentCommand::Branch
            )))
        );
    }

    fn create_branch_popup_model() -> Model {
        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Branch));
        model
    }

    #[test]
    fn test_m_in_branch_popup_shows_rename_select() {
        let model = create_branch_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('m'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowRenameBranchPopup));
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

    #[test]
    fn test_c_in_branch_popup_shows_checkout_new_branch_popup() {
        let model = create_branch_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('c'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ShowCreateNewBranchPopup { checkout: true })
        );
    }

    #[test]
    fn test_l_in_branch_popup_shows_checkout_local_branch_popup() {
        let model = create_branch_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('l'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowCheckoutLocalBranchPopup));
    }

    // Input popup tests

    fn create_input_popup_model() -> Model {
        use crate::model::popup::{InputContext, InputPopupState};

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Input(InputPopupState::new(
            "New branch name".to_string(),
            InputContext::CreateNewBranch {
                starting_point: "main".to_string(),
                checkout: true,
            },
        )));
        model
    }

    #[test]
    fn test_input_popup_esc_dismisses() {
        let model = create_input_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_input_popup_ctrl_g_dismisses() {
        let model = create_input_popup_model();

        let key = create_key_event(KeyModifiers::CONTROL, KeyCode::Char('g'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_input_popup_char_input() {
        use crate::msg::InputMessage;

        let model = create_input_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('a'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Input(InputMessage::InputChar('a'))));
    }

    #[test]
    fn test_input_popup_backspace() {
        use crate::msg::InputMessage;

        let model = create_input_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Backspace);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Input(InputMessage::InputBackspace)));
    }

    #[test]
    fn test_input_popup_enter_confirms() {
        use crate::msg::InputMessage;

        let model = create_input_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Enter);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Input(InputMessage::Confirm)));
    }

    // Fetch popup tests

    #[test]
    fn test_f_shows_fetch_popup() {
        let model = create_test_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('f'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowFetchPopup));
    }

    #[test]
    fn test_u_in_fetch_popup_with_upstream_fetches() {
        use crate::model::popup::FetchPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(
            FetchPopupState {
                upstream: Some("origin/main".to_string()),
            },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('u'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::FetchUpstream));
    }

    #[test]
    fn test_u_in_fetch_popup_without_upstream_shows_select() {
        use crate::model::popup::FetchPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(
            FetchPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('u'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowFetchUpstreamSelect));
    }

    #[test]
    fn test_a_in_fetch_popup_fetches_all_remotes() {
        use crate::model::popup::FetchPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(
            FetchPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('a'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::FetchAllRemotes));
    }

    #[test]
    fn test_esc_dismisses_fetch_popup() {
        use crate::model::popup::FetchPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(
            FetchPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_minus_in_fetch_popup_enters_arg_mode() {
        use crate::model::popup::FetchPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(
            FetchPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('-'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::EnterArgMode));
    }

    #[test]
    fn test_p_in_fetch_arg_mode_toggles_prune() {
        use crate::model::popup::FetchPopupState;

        let mut model = create_test_model();
        model.arg_mode = true;
        model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(
            FetchPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('p'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ToggleArgument(Fetch(FetchArgument::Prune)))
        );
    }

    #[test]
    fn test_t_in_fetch_arg_mode_toggles_tags() {
        use crate::model::popup::FetchPopupState;

        let mut model = create_test_model();
        model.arg_mode = true;
        model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(
            FetchPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('t'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ToggleArgument(Fetch(FetchArgument::Tags)))
        );
    }

    #[test]
    fn test_shift_f_in_fetch_arg_mode_toggles_force() {
        use crate::model::popup::FetchPopupState;

        let mut model = create_test_model();
        model.arg_mode = true;
        model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(
            FetchPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::SHIFT, KeyCode::Char('F'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ToggleArgument(Fetch(FetchArgument::Force)))
        );
    }

    // Pull popup tests

    #[test]
    fn test_shift_f_shows_pull_popup() {
        let model = create_test_model();

        let key = create_key_event(KeyModifiers::SHIFT, KeyCode::Char('F'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowPullPopup));
    }

    #[test]
    fn test_u_in_pull_popup_with_upstream_pulls() {
        use crate::model::popup::PullPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(
            PullPopupState {
                upstream: Some("origin/main".to_string()),
            },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('u'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::PullUpstream));
    }

    #[test]
    fn test_u_in_pull_popup_without_upstream_shows_select() {
        use crate::model::popup::PullPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(
            PullPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('u'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowPullUpstreamSelect));
    }

    #[test]
    fn test_esc_dismisses_pull_popup() {
        use crate::model::popup::PullPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(
            PullPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_minus_in_pull_popup_enters_arg_mode() {
        use crate::model::popup::PullPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(
            PullPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('-'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::EnterArgMode));
    }

    #[test]
    fn test_invalid_key_in_pull_arg_mode_exits_arg_mode() {
        use crate::model::popup::PullPopupState;

        let mut model = create_test_model();
        model.arg_mode = true;
        model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(
            PullPopupState { upstream: None },
        )));

        // 'r' is not a valid pull argument key, so it should exit arg mode
        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('w'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ExitArgMode));
    }

    #[test]
    fn test_f_in_pull_arg_mode_toggles_ff_only() {
        use crate::model::arguments::Argument::Pull;
        use crate::model::arguments::PullArgument;
        use crate::model::popup::PullPopupState;

        let mut model = create_test_model();
        model.arg_mode = true;
        model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(
            PullPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('f'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ToggleArgument(Pull(PullArgument::FfOnly)))
        );
    }

    #[test]
    fn test_r_in_pull_arg_mode_toggles_rebase() {
        use crate::model::arguments::Argument::Pull;
        use crate::model::arguments::PullArgument;
        use crate::model::popup::PullPopupState;

        let mut model = create_test_model();
        model.arg_mode = true;
        model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(
            PullPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('r'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ToggleArgument(Pull(PullArgument::Rebase)))
        );
    }

    #[test]
    fn test_a_in_pull_arg_mode_toggles_autostash() {
        use crate::model::arguments::Argument::Pull;
        use crate::model::arguments::PullArgument;
        use crate::model::popup::PullPopupState;

        let mut model = create_test_model();
        model.arg_mode = true;
        model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(
            PullPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('a'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ToggleArgument(Pull(PullArgument::Autostash)))
        );
    }

    #[test]
    fn test_f_in_pull_arg_mode_toggles_force() {
        use crate::model::arguments::Argument::Pull;
        use crate::model::arguments::PullArgument;
        use crate::model::popup::PullPopupState;

        let mut model = create_test_model();
        model.arg_mode = true;
        model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(
            PullPopupState { upstream: None },
        )));

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('F'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ToggleArgument(Pull(PullArgument::Force)))
        );
    }

    // Log popup tests

    #[test]
    fn test_l_shows_log_popup() {
        let model = create_test_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('l'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ShowPopup(PopupContent::Command(
                PopupContentCommand::Log
            )))
        );
    }

    fn create_log_popup_model() -> Model {
        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Log));
        model
    }

    #[test]
    fn test_l_in_log_popup_shows_log_view() {
        let model = create_log_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('l'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowLogCurrent));
    }

    #[test]
    fn test_esc_dismisses_log_popup() {
        let model = create_log_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_q_dismisses_log_popup() {
        let model = create_log_popup_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('q'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    // Log view mode tests - navigation uses same messages as status view

    fn create_log_mode_model() -> Model {
        use crate::model::ViewMode;

        let mut model = create_test_model();
        model.view_mode = ViewMode::Log;
        model
    }

    #[test]
    fn test_q_in_log_mode_exits_log_view() {
        let model = create_log_mode_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('q'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ExitLogView));
    }

    #[test]
    fn test_q_in_status_mode_quits() {
        let model = create_test_model(); // Default is Status mode

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('q'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Quit));
    }

    #[test]
    fn test_j_in_log_mode_moves_down() {
        // Navigation uses same messages in both modes
        let model = create_log_mode_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('j'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::MoveDown));
    }

    #[test]
    fn test_k_in_log_mode_moves_up() {
        let model = create_log_mode_model();

        let key = create_key_event(KeyModifiers::NONE, KeyCode::Char('k'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::MoveUp));
    }

    #[test]
    fn test_ctrl_d_in_log_mode_half_page_down() {
        let model = create_log_mode_model();

        let key = create_key_event(KeyModifiers::CONTROL, KeyCode::Char('d'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::HalfPageDown));
    }

    #[test]
    fn test_ctrl_u_in_log_mode_half_page_up() {
        let model = create_log_mode_model();

        let key = create_key_event(KeyModifiers::CONTROL, KeyCode::Char('u'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::HalfPageUp));
    }
}
