use magi::{
    git::{log::get_log_entries, test_repo::TestRepo},
    model::{ViewMode, popup::CommitSelectPopupState, select_popup::OnSelect},
    msg::{
        CommitSelect, FixupType, LogType, Message, NavigationAction, SelectMessage, update::update,
    },
};

mod utils;
use utils::create_model_from_test_repo;

/// Helper to get log entries for testing (filters out graph-only entries)
fn get_log_entries_for_test(test_repo: &TestRepo) -> Vec<magi::model::LogEntry> {
    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let mut entries = get_log_entries(&repo, LogType::Current).unwrap();
    entries.retain(|e| e.is_commit());
    entries
}

#[test]
fn test_commit_select_popup_displays_log_entries() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    test_repo
        .write_file_content("file2.txt", "content2")
        .stage_files(&["file2.txt"])
        .commit("Second commit");

    // Stage some changes to prepare for fixup
    test_repo
        .write_file_content("file1.txt", "modified content")
        .stage_files(&["file1.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);

    // Show the log pick view
    let _result = update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::FixupCommit(FixupType::Fixup)),
    );

    // Verify we switched to log pick mode (not a popup)
    assert!(
        matches!(model.view_mode, ViewMode::Log(LogType::Current, true)),
        "Expected log pick view, got {:?}",
        model.view_mode
    );
    assert!(model.popup.is_none());

    // Verify lines contain log entries
    assert!(model.ui_model.lines.len() >= 2);
    // Verify entries have proper structure (hash, message, etc.)
    if let magi::model::LineContent::LogLine(entry) = &model.ui_model.lines[0].content {
        assert!(entry.hash.is_some());
        assert!(entry.message.is_some());
    } else {
        panic!("Expected LogLine content");
    }
}

#[test]
fn test_commit_select_popup_confirm_returns_hash() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    test_repo
        .write_file_content("file2.txt", "content2")
        .stage_files(&["file2.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);

    // Show the log pick view
    let _result = update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::FixupCommit(FixupType::Fixup)),
    );

    assert!(matches!(
        model.view_mode,
        ViewMode::Log(LogType::Current, true)
    ));

    // Get the expected hash from the cursor line (position 0)
    let expected_hash =
        if let magi::model::LineContent::LogLine(entry) = &model.ui_model.lines[0].content {
            entry.hash.clone().unwrap()
        } else {
            panic!("Expected LogLine at cursor");
        };

    // Confirm selection
    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    // Verify view returned to status and correct message is returned
    assert_eq!(model.view_mode, ViewMode::Status);
    assert!(model.popup.is_none());
    assert_eq!(
        result,
        Some(Message::FixupCommit(expected_hash, FixupType::Fixup))
    );
}

#[test]
fn test_commit_select_popup_navigation() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("Commit 1");

    test_repo
        .write_file_content("file2.txt", "content2")
        .stage_files(&["file2.txt"])
        .commit("Commit 2");

    test_repo
        .write_file_content("file3.txt", "content3")
        .stage_files(&["file3.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);

    // Show the log pick view
    let _result = update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::FixupCommit(FixupType::Fixup)),
    );

    assert!(matches!(
        model.view_mode,
        ViewMode::Log(LogType::Current, true)
    ));
    assert_eq!(model.ui_model.cursor_position, 0);

    // Move down — uses standard MoveDown message
    update(&mut model, Message::Navigation(NavigationAction::MoveDown));
    assert_eq!(model.ui_model.cursor_position, 1);

    // Move up — uses standard MoveUp message
    update(&mut model, Message::Navigation(NavigationAction::MoveUp));
    assert_eq!(model.ui_model.cursor_position, 0);
}

#[test]
fn test_commit_select_popup_confirm_picks_second_commit() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    test_repo
        .write_file_content("file2.txt", "content2")
        .stage_files(&["file2.txt"])
        .commit("Second commit");

    test_repo
        .write_file_content("file3.txt", "content3")
        .stage_files(&["file3.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);

    update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::FixupCommit(FixupType::Squash)),
    );

    // Move to second commit (index 1)
    update(&mut model, Message::Navigation(NavigationAction::MoveDown));
    assert_eq!(model.ui_model.cursor_position, 1);

    let expected_hash =
        if let magi::model::LineContent::LogLine(entry) = &model.ui_model.lines[1].content {
            entry.hash.clone().unwrap()
        } else {
            panic!("Expected LogLine at index 1");
        };

    let result = update(&mut model, Message::Select(SelectMessage::Confirm));
    assert_eq!(model.view_mode, ViewMode::Status);
    assert_eq!(
        result,
        Some(Message::FixupCommit(expected_hash, FixupType::Squash))
    );
}

#[test]
fn test_commit_select_popup_escape_cancels() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    test_repo
        .write_file_content("file2.txt", "content2")
        .stage_files(&["file2.txt"]);

    let mut model = create_model_from_test_repo(&test_repo);

    update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::FixupCommit(FixupType::Fixup)),
    );

    assert!(matches!(model.view_mode, ViewMode::Log(_, true)));
    assert!(model.log_pick_on_select.is_some());

    // Escape via ExitLogView
    update(&mut model, Message::ExitLogView);

    assert_eq!(model.view_mode, ViewMode::Status);
    // log_pick_on_select should be cleared on cancel
    assert!(model.log_pick_on_select.is_none());
}

#[test]
fn test_rebase_elsewhere_opens_log_pick_all_references() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    test_repo
        .write_file_content("file2.txt", "content2")
        .stage_files(&["file2.txt"])
        .commit("Second commit");

    let mut model = create_model_from_test_repo(&test_repo);

    update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::RebaseElsewhere),
    );

    assert!(
        matches!(model.view_mode, ViewMode::Log(LogType::AllReferences, true)),
        "Expected AllReferences log pick view"
    );
    assert!(model.popup.is_none());
    assert_eq!(model.log_pick_on_select, Some(OnSelect::RebaseElsewhere));
}

#[test]
fn test_commit_select_popup_state_new() {
    let commits = get_log_entries_for_test(&TestRepo::new());
    let state = CommitSelectPopupState::new("Test".to_string(), commits.clone());

    assert_eq!(state.title, "Test");
    assert_eq!(state.all_commits, commits);
    assert_eq!(state.filtered_indices.len(), commits.len());
    assert_eq!(state.input_text, "");
    assert_eq!(state.selected_index, 0);
}

#[test]
fn test_commit_select_popup_state_filter() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("Feature X");

    test_repo
        .write_file_content("file2.txt", "content2")
        .stage_files(&["file2.txt"])
        .commit("Bug fix");

    let commits = get_log_entries_for_test(&test_repo);
    let mut state = CommitSelectPopupState::new("Test".to_string(), commits);

    // Filter by "feature"
    state.input_text = "feature".to_string();
    state.update_filter();

    assert_eq!(state.filtered_count(), 1);
    assert_eq!(
        state.selected_commit().unwrap().message.as_deref(),
        Some("Feature X")
    );
}
