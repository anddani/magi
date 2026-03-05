use crossterm::event::{
    self,
    KeyCode::{Backspace, Char, Down, Enter, Esc, Tab, Up},
    KeyModifiers,
};

use crate::{
    model::{
        Model, ViewMode,
        popup::{ConfirmAction, PopupContent, PopupContentCommand},
    },
    msg::{Message, NavigationAction, RebaseCommand, SearchMessage, SelectMessage},
};

mod command_popup;
mod credentials_popup;
mod input_popup;

const NONE: KeyModifiers = KeyModifiers::NONE;
const CTRL: KeyModifiers = KeyModifiers::CONTROL;

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
        'z' | 'Z' => Some(Message::ShowPopup(PopupContent::Command(
            PopupContentCommand::Stash,
        ))),
        'r' => Some(Message::ShowRebasePopup),
        '_' => Some(Message::ShowRevertPopup),
        'O' => Some(Message::ShowResetPopup),
        _ => None,
    }
}

/// Maps a key event into a [`Message`] given the application state.
/// If function returns [`None`], no action should be triggered.
pub fn handle_key(key: event::KeyEvent, model: &Model) -> Option<Message> {
    if let Some(PopupContent::Error { .. }) = &model.popup {
        return match key.code {
            Enter | Esc => Some(Message::DismissPopup),
            _ => None,
        };
    }

    if let Some(PopupContent::Confirm(state)) = &model.popup {
        return match (key.modifiers, key.code) {
            (_, Char('y')) | (_, Enter) => {
                let msg = match &state.on_confirm {
                    ConfirmAction::DeleteBranch(branch) => {
                        Message::ConfirmDeleteBranch(branch.clone())
                    }
                    ConfirmAction::DiscardChanges(target) => {
                        Message::ConfirmDiscard(target.clone())
                    }
                    ConfirmAction::PopStash(stash_ref) => {
                        Message::ConfirmPopStash(stash_ref.clone())
                    }
                    ConfirmAction::DropStash(stash_ref) => {
                        Message::ConfirmDropStash(stash_ref.clone())
                    }
                    ConfirmAction::RebaseElsewhere(target) => {
                        Message::Rebase(RebaseCommand::Elsewhere(target.clone()))
                    }
                    ConfirmAction::ResetBranch { branch, target } => Message::ResetBranch {
                        branch: branch.clone(),
                        target: target.clone(),
                    },
                };
                Some(msg)
            }
            (_, Char('n')) | (_, Esc) | (CTRL, Char('c')) | (CTRL, Char('g')) => {
                Some(Message::DismissPopup)
            }
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

    // Let commands from help popup open popups
    if model.popup == Some(PopupContent::Help) {
        return match (key.modifiers, key.code) {
            (_, Esc) | (_, Char('q')) | (CTRL, Char('g')) => Some(Message::DismissPopup),
            (_, Char(c)) => command_popup_keys(c),
            _ => None,
        };
    }

    // Check for visual mode exit keys first (ESC and Ctrl-g)
    if model.ui_model.is_visual_mode() {
        match (key.modifiers, key.code) {
            (NONE, Esc) | (CTRL, Char('g')) | (CTRL, Char('c')) => {
                return Some(Message::ExitVisualMode);
            }
            // Disable ToggleSection in visual mode to prevent confusing selection behavior
            (NONE, Tab) => {
                return None;
            }
            _ => {}
        }
    }

    // Handle pending 'g' for 'gg' (go to first line)
    if model.pending_g {
        if key.modifiers == NONE && key.code == Char('g') {
            return Some(Message::Navigation(NavigationAction::MoveToTop));
        }
        if key.modifiers == NONE && key.code == Char('r') {
            return Some(Message::Refresh);
        }
        // Any other key cancels the pending 'g' and falls through to normal handling
    }

    // Handle search input mode — route keystrokes to search handler
    if model.ui_model.search_mode_active {
        return match (key.modifiers, key.code) {
            (_, Esc) | (CTRL, Char('g')) | (CTRL, Char('c')) => {
                Some(Message::Search(SearchMessage::Cancel))
            }
            (_, Enter) => Some(Message::Search(SearchMessage::Confirm)),
            (_, Backspace) => Some(Message::Search(SearchMessage::InputBackspace)),
            // Allow navigation during typing
            (CTRL, Char('u')) => Some(Message::Navigation(NavigationAction::HalfPageUp)),
            (CTRL, Char('d')) => Some(Message::Navigation(NavigationAction::HalfPageDown)),
            (CTRL, Char('e')) => Some(Message::Navigation(NavigationAction::ScrollLineDown)),
            (CTRL, Char('y')) => Some(Message::Navigation(NavigationAction::ScrollLineUp)),
            (_, Up) => Some(Message::Navigation(NavigationAction::MoveUp)),
            (_, Down) => Some(Message::Navigation(NavigationAction::MoveDown)),
            // All other chars go into the search field
            (_, Char(c)) => Some(Message::Search(SearchMessage::InputChar(c))),
            _ => None,
        };
    }

    // Cancel active search with Esc (when not in search input mode)
    if !model.ui_model.search_query.is_empty()
        && matches!((key.modifiers, key.code), (_, Esc) | (CTRL, Char('g')))
    {
        return Some(Message::Search(SearchMessage::Cancel));
    }

    // Enter/Esc in log pick mode
    if let ViewMode::Log(_, true) = model.view_mode {
        match (key.modifiers, key.code) {
            (_, Enter) => return Some(Message::Select(SelectMessage::Confirm)),
            (_, Esc) | (CTRL, Char('g')) | (CTRL, Char('c')) => {
                return Some(Message::ExitLogView);
            }
            _ => {}
        }
    }

    match (key.modifiers, key.code) {
        // Navigation
        (CTRL, Char('u')) => Some(Message::Navigation(NavigationAction::HalfPageUp)),
        (CTRL, Char('d')) => Some(Message::Navigation(NavigationAction::HalfPageDown)),
        (CTRL, Char('e')) => Some(Message::Navigation(NavigationAction::ScrollLineDown)),
        (CTRL, Char('y')) => Some(Message::Navigation(NavigationAction::ScrollLineUp)),
        (_, Char('k') | Up) => Some(Message::Navigation(NavigationAction::MoveUp)),
        (_, Char('j') | Down) => Some(Message::Navigation(NavigationAction::MoveDown)),
        (_, Char('G')) => Some(Message::Navigation(NavigationAction::MoveToBottom)),

        // General actions
        (CTRL, Char('r')) => Some(Message::Refresh),
        (NONE, Char('g')) => Some(Message::PendingG),
        (_, Tab) => Some(Message::ToggleSection),
        (_, Char('?') | Char('h')) => Some(Message::ShowPopup(PopupContent::Help)),
        (_, Char('q')) => match model.view_mode {
            ViewMode::Log(_, _) => Some(Message::ExitLogView),
            ViewMode::Status => Some(Message::Quit),
        },
        (_, Char('V')) => Some(Message::EnterVisualMode),
        (_, Char('s')) => Some(Message::StageSelected),
        (_, Char('S')) => Some(Message::StageAllModified),
        (_, Char('u')) => Some(Message::UnstageSelected),
        (_, Char('U')) => Some(Message::UnstageAll),
        (_, Char('x')) => Some(Message::DiscardSelected),

        // Search
        (NONE, Char('/')) => Some(Message::EnterSearchMode),
        (NONE, Char('n')) => Some(Message::Search(SearchMessage::Next)),
        (KeyModifiers::SHIFT, Char('N')) => Some(Message::Search(SearchMessage::Prev)),

        (_, Char(c)) => command_popup_keys(c),
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
    use crate::msg::{FetchCommand, NavigationAction, PullCommand, PushCommand, SelectMessage};
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
        let workdir = repo_path.to_path_buf();
        Model {
            git_info,
            workdir,
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
            cursor_reposition_context: None,
        }
    }

    #[test]
    fn test_shift_v_enters_visual_mode() {
        let model = create_test_model();
        let key = create_key_event(KeyModifiers::SHIFT, Char('V'));

        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::EnterVisualMode));
    }

    #[test]
    fn test_esc_exits_visual_mode() {
        let mut model = create_test_model();
        model.ui_model.visual_mode_anchor = Some(5); // Enter visual mode

        let key = create_key_event(NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ExitVisualMode));
    }

    #[test]
    fn test_ctrl_g_exits_visual_mode() {
        let mut model = create_test_model();
        model.ui_model.visual_mode_anchor = Some(5); // Enter visual mode

        let key = create_key_event(CTRL, Char('g'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ExitVisualMode));
    }

    #[test]
    fn test_esc_does_nothing_when_not_in_visual_mode() {
        let model = create_test_model();
        let key = create_key_event(NONE, KeyCode::Esc);

        let result = handle_key(key, &model);
        assert_eq!(result, None);
    }

    #[test]
    fn test_ctrl_g_does_nothing_when_not_in_visual_mode() {
        let model = create_test_model();
        let key = create_key_event(CTRL, Char('g'));

        let result = handle_key(key, &model);
        assert_eq!(result, None);
    }

    #[test]
    fn test_movement_keys_work_in_visual_mode() {
        let mut model = create_test_model();
        model.ui_model.visual_mode_anchor = Some(5); // Enter visual mode

        // j should still work
        let key = create_key_event(NONE, Char('j'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::Navigation(NavigationAction::MoveDown))
        );

        // k should still work
        let key = create_key_event(NONE, Char('k'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Navigation(NavigationAction::MoveUp)));
    }

    #[test]
    fn test_tab_disabled_in_visual_mode() {
        let mut model = create_test_model();
        model.ui_model.visual_mode_anchor = Some(5); // Enter visual mode

        let key = create_key_event(NONE, KeyCode::Tab);
        let result = handle_key(key, &model);
        assert_eq!(result, None); // Tab should do nothing in visual mode
    }

    #[test]
    fn test_tab_works_when_not_in_visual_mode() {
        let model = create_test_model();

        let key = create_key_event(NONE, KeyCode::Tab);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ToggleSection));
    }

    #[test]
    fn test_question_mark_shows_help() {
        let model = create_test_model();

        let key = create_key_event(NONE, Char('?'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowPopup(PopupContent::Help)));
    }

    #[test]
    fn test_question_mark_does_nothing_when_error_popup_shown() {
        let mut model = create_test_model();
        model.popup = Some(PopupContent::Error {
            message: "Test error".to_string(),
        });

        let key = create_key_event(NONE, Char('?'));
        let result = handle_key(key, &model);
        // When error popup is shown, only Enter/Esc should work
        assert_eq!(result, None);
    }

    #[test]
    fn test_esc_dismisses_help_popup() {
        let mut model = create_test_model();
        model.popup = Some(PopupContent::Help);

        let key = create_key_event(NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_q_dismisses_help_popup() {
        let mut model = create_test_model();
        model.popup = Some(PopupContent::Help);

        let key = create_key_event(NONE, Char('q'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_c_shows_commit_popup() {
        let model = create_test_model();

        let key = create_key_event(NONE, Char('c'));
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

        let key = create_key_event(NONE, Char('c'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Commit));
    }

    #[test]
    fn test_esc_dismisses_commit_popup() {
        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Commit));

        let key = create_key_event(NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_q_dismisses_commit_popup() {
        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Commit));

        let key = create_key_event(NONE, Char('q'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_a_in_commit_popup_triggers_amend() {
        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Commit));

        let key = create_key_event(NONE, Char('a'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Amend(vec![])));
    }

    #[test]
    fn test_shift_p_shows_push_popup() {
        let model = create_test_model();

        let key = create_key_event(KeyModifiers::SHIFT, Char('P'));
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
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('u'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Push(PushCommand::PushUpstream)));
    }

    #[test]
    fn test_u_in_push_popup_without_upstream_shows_select() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('u'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ShowSelectPopup(
                crate::msg::SelectPopup::PushUpstream
            ))
        );
    }

    #[test]
    fn test_t_in_push_popup_shows_push_all_tags_select() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('t'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ShowSelectPopup(
                crate::msg::SelectPopup::PushAllTags
            ))
        );
    }

    #[test]
    fn test_shift_t_in_push_popup_shows_push_tag_select() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(KeyModifiers::SHIFT, Char('T'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ShowSelectPopup(crate::msg::SelectPopup::PushTag))
        );
    }

    #[test]
    fn test_esc_dismisses_push_popup() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_p_in_push_popup_with_push_remote_pushes_to_push_remote() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                upstream: None,
                push_remote: Some("origin".to_string()),
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('p'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::Push(PushCommand::PushToPushRemote(
                "origin".to_string()
            )))
        );
    }

    #[test]
    fn test_p_in_push_popup_without_push_remote_shows_select() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('p'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ShowSelectPopup(
                crate::msg::SelectPopup::PushPushRemote
            ))
        );
    }

    #[test]
    fn test_p_in_push_popup_with_sole_remote_pushes_directly() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: Some("origin".to_string()),
            },
        )));

        let key = create_key_event(NONE, Char('p'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::Push(PushCommand::PushToPushRemote(
                "origin".to_string()
            )))
        );
    }

    #[test]
    fn test_minus_in_push_popup_enters_arg_mode() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('-'));
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
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('f'));
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
            PushPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('x'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ExitArgMode));
    }

    #[test]
    fn test_esc_in_arg_mode_dismisses_popup() {
        use crate::model::popup::PushPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Push(
            PushPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, KeyCode::Esc);
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

        let key = create_key_event(NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_select_popup_ctrl_g_dismisses() {
        let model = create_select_popup_model();

        let key = create_key_event(CTRL, Char('g'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_select_popup_enter_confirms() {
        let model = create_select_popup_model();

        let key = create_key_event(NONE, KeyCode::Enter);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Select(SelectMessage::Confirm)));
    }

    #[test]
    fn test_select_popup_char_input() {
        let model = create_select_popup_model();

        let key = create_key_event(NONE, Char('a'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Select(SelectMessage::InputChar('a'))));
    }

    #[test]
    fn test_select_popup_shift_char_input() {
        let model = create_select_popup_model();

        let key = create_key_event(KeyModifiers::SHIFT, Char('A'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Select(SelectMessage::InputChar('A'))));
    }

    #[test]
    fn test_select_popup_backspace() {
        let model = create_select_popup_model();

        let key = create_key_event(NONE, KeyCode::Backspace);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Select(SelectMessage::InputBackspace)));
    }

    #[test]
    fn test_select_popup_up_arrow() {
        let model = create_select_popup_model();

        let key = create_key_event(NONE, KeyCode::Up);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Select(SelectMessage::MoveUp)));
    }

    #[test]
    fn test_select_popup_down_arrow() {
        let model = create_select_popup_model();

        let key = create_key_event(NONE, KeyCode::Down);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Select(SelectMessage::MoveDown)));
    }

    #[test]
    fn test_select_popup_ctrl_p_moves_up() {
        let model = create_select_popup_model();

        let key = create_key_event(CTRL, Char('p'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Select(SelectMessage::MoveUp)));
    }

    #[test]
    fn test_select_popup_ctrl_n_moves_down() {
        let model = create_select_popup_model();

        let key = create_key_event(CTRL, Char('n'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Select(SelectMessage::MoveDown)));
    }

    // Branch popup tests

    #[test]
    fn test_b_shows_branch_popup() {
        let model = create_test_model();

        let key = create_key_event(NONE, Char('b'));
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

        let key = create_key_event(NONE, Char('m'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ShowSelectPopup(
                crate::msg::SelectPopup::RenameBranch
            ))
        );
    }

    #[test]
    fn test_esc_dismisses_branch_popup() {
        let model = create_branch_popup_model();

        let key = create_key_event(NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_q_dismisses_branch_popup() {
        let model = create_branch_popup_model();

        let key = create_key_event(NONE, Char('q'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_b_in_branch_popup_shows_checkout_select() {
        let model = create_branch_popup_model();

        let key = create_key_event(NONE, Char('b'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ShowSelectPopup(
                crate::msg::SelectPopup::CheckoutBranch
            ))
        );
    }

    #[test]
    fn test_c_in_branch_popup_shows_checkout_new_branch_popup() {
        let model = create_branch_popup_model();

        let key = create_key_event(NONE, Char('c'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ShowSelectPopup(
                crate::msg::SelectPopup::CreateNewBranch { checkout: true }
            ))
        );
    }

    #[test]
    fn test_l_in_branch_popup_shows_checkout_local_branch_popup() {
        let model = create_branch_popup_model();

        let key = create_key_event(NONE, Char('l'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ShowSelectPopup(
                crate::msg::SelectPopup::CheckoutLocalBranch
            ))
        );
    }

    // Input popup tests

    fn create_input_popup_model() -> Model {
        use crate::model::popup::InputContext;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::input_popup(InputContext::CreateNewBranch {
            starting_point: "main".to_string(),
            checkout: true,
        }));
        model
    }

    #[test]
    fn test_input_popup_esc_dismisses() {
        let model = create_input_popup_model();

        let key = create_key_event(NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_input_popup_ctrl_g_dismisses() {
        let model = create_input_popup_model();

        let key = create_key_event(CTRL, Char('g'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_input_popup_char_input() {
        use crate::msg::InputMessage;

        let model = create_input_popup_model();

        let key = create_key_event(NONE, Char('a'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Input(InputMessage::InputChar('a'))));
    }

    #[test]
    fn test_input_popup_backspace() {
        use crate::msg::InputMessage;

        let model = create_input_popup_model();

        let key = create_key_event(NONE, KeyCode::Backspace);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Input(InputMessage::InputBackspace)));
    }

    #[test]
    fn test_input_popup_enter_confirms() {
        use crate::msg::InputMessage;

        let model = create_input_popup_model();

        let key = create_key_event(NONE, KeyCode::Enter);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Input(InputMessage::Confirm)));
    }

    // Fetch popup tests

    #[test]
    fn test_f_shows_fetch_popup() {
        let model = create_test_model();

        let key = create_key_event(NONE, Char('f'));
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
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('u'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Fetch(FetchCommand::FetchUpstream)));
    }

    #[test]
    fn test_u_in_fetch_popup_without_upstream_shows_select() {
        use crate::model::popup::FetchPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(
            FetchPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('u'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ShowSelectPopup(
                crate::msg::SelectPopup::FetchUpstream
            ))
        );
    }

    #[test]
    fn test_a_in_fetch_popup_fetches_all_remotes() {
        use crate::model::popup::FetchPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(
            FetchPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('a'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Fetch(FetchCommand::FetchAllRemotes)));
    }

    #[test]
    fn test_esc_dismisses_fetch_popup() {
        use crate::model::popup::FetchPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(
            FetchPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_minus_in_fetch_popup_enters_arg_mode() {
        use crate::model::popup::FetchPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(
            FetchPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('-'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::EnterArgMode));
    }

    #[test]
    fn test_p_in_fetch_arg_mode_toggles_prune() {
        use crate::model::popup::FetchPopupState;

        let mut model = create_test_model();
        model.arg_mode = true;
        model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(
            FetchPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('p'));
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
            FetchPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('t'));
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
            FetchPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(KeyModifiers::SHIFT, Char('F'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ToggleArgument(Fetch(FetchArgument::Force)))
        );
    }

    #[test]
    fn test_p_in_fetch_popup_with_push_remote_fetches_from_push_remote() {
        use crate::model::popup::FetchPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(
            FetchPopupState {
                upstream: None,
                push_remote: Some("origin".to_string()),
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('p'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::Fetch(FetchCommand::FetchFromPushRemote(
                "origin".to_string()
            )))
        );
    }

    #[test]
    fn test_p_in_fetch_popup_without_push_remote_shows_select() {
        use crate::model::popup::FetchPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(
            FetchPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('p'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ShowSelectPopup(
                crate::msg::SelectPopup::FetchPushRemote
            ))
        );
    }

    #[test]
    fn test_p_in_fetch_popup_with_sole_remote_fetches_directly() {
        use crate::model::popup::FetchPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Fetch(
            FetchPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: Some("origin".to_string()),
            },
        )));

        let key = create_key_event(NONE, Char('p'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::Fetch(FetchCommand::FetchFromPushRemote(
                "origin".to_string()
            )))
        );
    }

    // Pull popup tests

    #[test]
    fn test_shift_f_shows_pull_popup() {
        let model = create_test_model();

        let key = create_key_event(KeyModifiers::SHIFT, Char('F'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowPullPopup));
    }

    #[test]
    fn test_p_in_pull_popup_with_push_remote_pulls_from_push_remote() {
        use crate::model::popup::PullPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(
            PullPopupState {
                upstream: None,
                push_remote: Some("origin".to_string()),
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('p'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::Pull(PullCommand::PullFromPushRemote(
                "origin".to_string()
            )))
        );
    }

    #[test]
    fn test_p_in_pull_popup_without_push_remote_shows_select() {
        use crate::model::popup::PullPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(
            PullPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('p'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ShowSelectPopup(
                crate::msg::SelectPopup::PullPushRemote
            ))
        );
    }

    #[test]
    fn test_p_in_pull_popup_with_sole_remote_pulls_directly() {
        use crate::model::popup::PullPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(
            PullPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: Some("origin".to_string()),
            },
        )));

        let key = create_key_event(NONE, Char('p'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::Pull(PullCommand::PullFromPushRemote(
                "origin".to_string()
            )))
        );
    }

    #[test]
    fn test_u_in_pull_popup_with_upstream_pulls() {
        use crate::model::popup::PullPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(
            PullPopupState {
                upstream: Some("origin/main".to_string()),
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('u'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Pull(PullCommand::PullUpstream)));
    }

    #[test]
    fn test_u_in_pull_popup_without_upstream_shows_select() {
        use crate::model::popup::PullPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(
            PullPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('u'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::ShowSelectPopup(
                crate::msg::SelectPopup::PullUpstream
            ))
        );
    }

    #[test]
    fn test_esc_dismisses_pull_popup() {
        use crate::model::popup::PullPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(
            PullPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_minus_in_pull_popup_enters_arg_mode() {
        use crate::model::popup::PullPopupState;

        let mut model = create_test_model();
        model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(
            PullPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('-'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::EnterArgMode));
    }

    #[test]
    fn test_invalid_key_in_pull_arg_mode_exits_arg_mode() {
        use crate::model::popup::PullPopupState;

        let mut model = create_test_model();
        model.arg_mode = true;
        model.popup = Some(PopupContent::Command(PopupContentCommand::Pull(
            PullPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        // 'r' is not a valid pull argument key, so it should exit arg mode
        let key = create_key_event(NONE, Char('w'));
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
            PullPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('f'));
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
            PullPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('r'));
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
            PullPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('a'));
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
            PullPopupState {
                upstream: None,
                push_remote: None,
                sole_remote: None,
            },
        )));

        let key = create_key_event(NONE, Char('F'));
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

        let key = create_key_event(NONE, Char('l'));
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
        use crate::msg::LogType;

        let model = create_log_popup_model();

        let key = create_key_event(NONE, Char('l'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowLog(LogType::Current)));
    }

    #[test]
    fn test_a_in_log_popup_shows_all_refs_log_view() {
        use crate::msg::LogType;

        let model = create_log_popup_model();

        let key = create_key_event(NONE, Char('a'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ShowLog(LogType::AllReferences)));
    }

    #[test]
    fn test_esc_dismisses_log_popup() {
        let model = create_log_popup_model();

        let key = create_key_event(NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_q_dismisses_log_popup() {
        let model = create_log_popup_model();

        let key = create_key_event(NONE, Char('q'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    // Log view mode tests - navigation uses same messages as status view

    fn create_log_mode_model() -> Model {
        use crate::model::ViewMode;
        use crate::msg::LogType;

        let mut model = create_test_model();
        model.view_mode = ViewMode::Log(LogType::Current, false);
        model
    }

    #[test]
    fn test_q_in_log_mode_exits_log_view() {
        let model = create_log_mode_model();

        let key = create_key_event(NONE, Char('q'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ExitLogView));
    }

    #[test]
    fn test_q_in_status_mode_quits() {
        let model = create_test_model(); // Default is Status mode

        let key = create_key_event(NONE, Char('q'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Quit));
    }

    #[test]
    fn test_j_in_log_mode_moves_down() {
        // Navigation uses same messages in both modes
        let model = create_log_mode_model();

        let key = create_key_event(NONE, Char('j'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::Navigation(NavigationAction::MoveDown))
        );
    }

    #[test]
    fn test_k_in_log_mode_moves_up() {
        let model = create_log_mode_model();

        let key = create_key_event(NONE, Char('k'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Navigation(NavigationAction::MoveUp)));
    }

    #[test]
    fn test_ctrl_d_in_log_mode_half_page_down() {
        let model = create_log_mode_model();

        let key = create_key_event(CTRL, Char('d'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::Navigation(NavigationAction::HalfPageDown))
        );
    }

    #[test]
    fn test_ctrl_u_in_log_mode_half_page_up() {
        let model = create_log_mode_model();

        let key = create_key_event(CTRL, Char('u'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::Navigation(NavigationAction::HalfPageUp))
        );
    }

    // Log pick mode tests

    fn create_log_pick_mode_model() -> Model {
        use crate::model::ViewMode;
        use crate::msg::LogType;

        let mut model = create_test_model();
        model.view_mode = ViewMode::Log(LogType::Current, true);
        model
    }

    #[test]
    fn test_enter_in_log_pick_mode_confirms_selection() {
        let model = create_log_pick_mode_model();

        let key = create_key_event(NONE, KeyCode::Enter);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Select(SelectMessage::Confirm)));
    }

    #[test]
    fn test_esc_in_log_pick_mode_exits_log_view() {
        let model = create_log_pick_mode_model();

        let key = create_key_event(NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ExitLogView));
    }

    #[test]
    fn test_ctrl_g_in_log_pick_mode_exits_log_view() {
        let model = create_log_pick_mode_model();

        let key = create_key_event(CTRL, Char('g'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ExitLogView));
    }

    #[test]
    fn test_q_in_log_pick_mode_exits_log_view() {
        let model = create_log_pick_mode_model();

        let key = create_key_event(NONE, Char('q'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ExitLogView));
    }

    #[test]
    fn test_j_in_log_pick_mode_moves_down() {
        let model = create_log_pick_mode_model();

        let key = create_key_event(NONE, Char('j'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::Navigation(NavigationAction::MoveDown))
        );
    }

    #[test]
    fn test_enter_in_browse_log_mode_does_nothing() {
        let model = create_log_mode_model(); // picking = false

        let key = create_key_event(NONE, KeyCode::Enter);
        let result = handle_key(key, &model);
        assert_eq!(result, None);
    }

    #[test]
    fn test_esc_in_browse_log_mode_does_nothing() {
        let model = create_log_mode_model(); // picking = false

        let key = create_key_event(NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, None);
    }

    // Discard tests

    #[test]
    fn test_x_triggers_discard_selected() {
        let model = create_test_model();

        let key = create_key_event(NONE, Char('x'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DiscardSelected));
    }

    #[test]
    fn test_y_in_discard_confirm_popup_triggers_confirm_discard() {
        use crate::model::popup::{ConfirmAction, ConfirmPopupState};
        use crate::msg::{DiscardSource, DiscardTarget};

        let mut model = create_test_model();
        let target = DiscardTarget::Files {
            paths: vec!["test.txt".to_string()],
            source: DiscardSource::Unstaged,
        };
        model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
            message: "Discard changes in test.txt?".to_string(),
            on_confirm: ConfirmAction::DiscardChanges(target.clone()),
        }));

        let key = create_key_event(NONE, Char('y'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::ConfirmDiscard(target)));
    }

    #[test]
    fn test_n_in_discard_confirm_popup_dismisses() {
        use crate::model::popup::{ConfirmAction, ConfirmPopupState};
        use crate::msg::{DiscardSource, DiscardTarget};

        let mut model = create_test_model();
        let target = DiscardTarget::Files {
            paths: vec!["test.txt".to_string()],
            source: DiscardSource::Unstaged,
        };
        model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
            message: "Discard changes in test.txt?".to_string(),
            on_confirm: ConfirmAction::DiscardChanges(target),
        }));

        let key = create_key_event(NONE, Char('n'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    #[test]
    fn test_esc_in_discard_confirm_popup_dismisses() {
        use crate::model::popup::{ConfirmAction, ConfirmPopupState};
        use crate::msg::{DiscardSource, DiscardTarget};

        let mut model = create_test_model();
        let target = DiscardTarget::Files {
            paths: vec!["test.txt".to_string()],
            source: DiscardSource::Unstaged,
        };
        model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
            message: "Discard changes in test.txt?".to_string(),
            on_confirm: ConfirmAction::DiscardChanges(target),
        }));

        let key = create_key_event(NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::DismissPopup));
    }

    // Search mode tests

    #[test]
    fn test_slash_enters_search_mode() {
        let model = create_test_model();

        let key = create_key_event(NONE, Char('/'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::EnterSearchMode));
    }

    #[test]
    fn test_search_mode_char_goes_to_search_input() {
        use crate::msg::SearchMessage;

        let mut model = create_test_model();
        model.ui_model.search_mode_active = true;

        let key = create_key_event(NONE, Char('a'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Search(SearchMessage::InputChar('a'))));
    }

    #[test]
    fn test_search_mode_backspace_deletes() {
        use crate::msg::SearchMessage;

        let mut model = create_test_model();
        model.ui_model.search_mode_active = true;
        model.ui_model.search_query = "foo".to_string();

        let key = create_key_event(NONE, KeyCode::Backspace);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Search(SearchMessage::InputBackspace)));
    }

    #[test]
    fn test_search_mode_enter_confirms() {
        use crate::msg::SearchMessage;

        let mut model = create_test_model();
        model.ui_model.search_mode_active = true;

        let key = create_key_event(NONE, KeyCode::Enter);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Search(SearchMessage::Confirm)));
    }

    #[test]
    fn test_search_mode_esc_cancels() {
        use crate::msg::SearchMessage;

        let mut model = create_test_model();
        model.ui_model.search_mode_active = true;

        let key = create_key_event(NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Search(SearchMessage::Cancel)));
    }

    #[test]
    fn test_search_mode_navigation_still_works() {
        let mut model = create_test_model();
        model.ui_model.search_mode_active = true;

        let key = create_key_event(CTRL, Char('u'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::Navigation(NavigationAction::HalfPageUp))
        );

        let key = create_key_event(CTRL, Char('d'));
        let result = handle_key(key, &model);
        assert_eq!(
            result,
            Some(Message::Navigation(NavigationAction::HalfPageDown))
        );
    }

    #[test]
    fn test_n_triggers_search_next() {
        use crate::msg::SearchMessage;

        let model = create_test_model();

        let key = create_key_event(NONE, Char('n'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Search(SearchMessage::Next)));
    }

    #[test]
    fn test_shift_n_triggers_search_prev() {
        use crate::msg::SearchMessage;

        let model = create_test_model();

        let key = create_key_event(KeyModifiers::SHIFT, Char('N'));
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Search(SearchMessage::Prev)));
    }

    #[test]
    fn test_esc_with_active_search_cancels() {
        use crate::msg::SearchMessage;

        let mut model = create_test_model();
        model.ui_model.search_query = "foo".to_string();

        let key = create_key_event(NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, Some(Message::Search(SearchMessage::Cancel)));
    }

    #[test]
    fn test_esc_without_active_search_does_nothing() {
        let model = create_test_model();

        let key = create_key_event(NONE, KeyCode::Esc);
        let result = handle_key(key, &model);
        assert_eq!(result, None);
    }
}
