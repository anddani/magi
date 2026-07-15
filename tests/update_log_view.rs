use magi::{
    git::test_repo::TestRepo,
    model::ViewMode,
    msg::{CommitSelect, FixupType, LogType, Message, SelectMessage, update::update},
};

mod utils;
use utils::create_model_from_test_repo;

// ── ExitLogView restores the Status view UI state ─────────────────────────────

#[test]
fn test_exit_log_view_restores_status_cursor_and_scroll() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    test_repo.commit_file("file2.txt", "content2", "Second commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Simulate a scrolled-down status view
    model.ui_model.cursor_position = 3;
    model.ui_model.scroll_offset = 2;

    // Enter log view — cursor and scroll reset for the log
    update(&mut model, Message::ShowLog(LogType::Current));
    assert!(matches!(model.view_mode, ViewMode::Log { picking: false, .. }));
    assert_eq!(model.ui_model.cursor_position, 0);
    assert_eq!(model.ui_model.scroll_offset, 0);

    // Scroll around in the log view
    model.ui_model.cursor_position = 1;
    model.ui_model.scroll_offset = 1;

    // Exit back to status — the previous UI state is restored
    let result = update(&mut model, Message::ExitLogView);
    assert_eq!(result, Some(Message::Refresh));
    assert_eq!(model.view_mode, ViewMode::Status);
    assert_eq!(model.ui_model.cursor_position, 3);
    assert_eq!(model.ui_model.scroll_offset, 2);
}

// ── Switching log types keeps the saved Status state ──────────────────────────

#[test]
fn test_switching_log_type_preserves_saved_status_state() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.ui_model.cursor_position = 2;
    model.ui_model.scroll_offset = 1;

    // Enter log view, then switch to another log type from within the log view
    update(&mut model, Message::ShowLog(LogType::Current));
    model.ui_model.cursor_position = 5;
    update(&mut model, Message::ShowLog(LogType::AllReferences));

    // Exiting still restores the original status state
    update(&mut model, Message::ExitLogView);
    assert_eq!(model.ui_model.cursor_position, 2);
    assert_eq!(model.ui_model.scroll_offset, 1);
}

// ── Confirming a log pick restores the Status view UI state ───────────────────

#[test]
fn test_log_pick_confirm_restores_status_cursor_and_scroll() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");
    test_repo
        .write_file_content("file2.txt", "content2")
        .stage_files(&["file2.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);
    model.ui_model.cursor_position = 3;
    model.ui_model.scroll_offset = 2;

    // Enter log pick mode
    update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::FixupCommit(FixupType::Fixup)),
    );
    assert!(matches!(model.view_mode, ViewMode::Log { picking: true, .. }));
    assert_eq!(model.ui_model.cursor_position, 0);
    assert_eq!(model.ui_model.scroll_offset, 0);

    // Confirm the selection — back to status with the previous UI state
    update(&mut model, Message::Select(SelectMessage::Confirm));
    assert_eq!(model.view_mode, ViewMode::Status);
    assert_eq!(model.ui_model.cursor_position, 3);
    assert_eq!(model.ui_model.scroll_offset, 2);
}
