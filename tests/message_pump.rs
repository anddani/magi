use magi::{
    git::test_repo::TestRepo,
    magi::process_messages,
    model::{
        RunningState,
        popup::{PopupContent, PopupContentCommand},
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::{MergeCommand, Message, SelectMessage},
};

mod utils;
use utils::create_model_from_test_repo;

/// Confirming a branch in the merge select popup emits
/// `Message::Merge(MergeCommand::Branch(_))` from within the update chain
/// (not from a key press). The message pump must still recognise it as an
/// external command and suspend the TUI instead of processing it inline —
/// otherwise the editor for the merge commit message can never be shown.
#[test]
fn test_select_confirm_merge_branch_suspends_tui() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Merge branch".to_string(),
            vec!["feature".to_string()],
            OnSelect::MergeElsewhere,
        ),
    )));

    process_messages(&mut model, Some(Message::Select(SelectMessage::Confirm)));

    assert_eq!(
        model.running_state,
        RunningState::LaunchExternalCommand(Message::Merge(MergeCommand::Branch(
            "feature".to_string()
        )))
    );
    assert!(model.pty_state.is_none());
}

/// Same as above but for "Merge and edit message": the confirmed branch must
/// route to `MergeCommand::EditMessage(_)` and suspend the TUI, since the
/// `--edit` flag always opens the editor.
#[test]
fn test_select_confirm_merge_edit_message_suspends_tui() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Merge branch (edit message)".to_string(),
            vec!["feature".to_string()],
            OnSelect::MergeEditMessage,
        ),
    )));

    process_messages(&mut model, Some(Message::Select(SelectMessage::Confirm)));

    assert_eq!(
        model.running_state,
        RunningState::LaunchExternalCommand(Message::Merge(MergeCommand::EditMessage(
            "feature".to_string()
        )))
    );
    assert!(model.pty_state.is_none());
}

#[test]
fn test_process_messages_drains_non_external_chain() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);

    process_messages(&mut model, Some(Message::Refresh));

    assert_eq!(model.running_state, RunningState::Running);
}
