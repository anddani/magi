use magi::{
    git::{log::get_log_entries, test_repo::TestRepo},
    model::{
        Line, LineContent, ViewMode,
        log_view::LogEntry,
        popup::{ConfirmAction, ConfirmPopupState, PopupContent},
        select_popup::OnSelect,
    },
    msg::{CommitSelect, LogType, Message, update::update},
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

fn make_log_line(hash: &str, message: &str) -> Line {
    Line {
        content: LineContent::LogLine(LogEntry {
            hash: Some(hash.to_string()),
            message: Some(message.to_string()),
            author: None,
            time: None,
            refs: vec![],
            graph: String::new(),
        }),
        section: None,
    }
}

#[test]
fn test_show_revise_commit_cursor_on_commit_shows_confirm() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Place cursor on a log line with a commit hash
    model.ui_model.lines = vec![make_log_line("abc1234", "First commit")];
    model.ui_model.cursor_position = 0;

    let result = update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::ReviseCommit),
    );

    assert_eq!(result, None);
    assert!(
        matches!(
            &model.popup,
            Some(PopupContent::Confirm(ConfirmPopupState {
                on_confirm: ConfirmAction::ReviseCommit(_),
                ..
            }))
        ),
        "Expected confirmation popup with ReviseCommit action"
    );

    if let Some(PopupContent::Confirm(state)) = &model.popup {
        assert!(
            state.message.contains("abc1234"),
            "Confirm message should contain hash"
        );
        assert_eq!(
            state.on_confirm,
            ConfirmAction::ReviseCommit("abc1234".to_string())
        );
    }
}

#[test]
fn test_show_revise_commit_cursor_not_on_commit_shows_log_pick() {
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
    // cursor on a non-commit line (empty)
    model.ui_model.lines = vec![Line {
        content: LineContent::EmptyLine,
        section: None,
    }];
    model.ui_model.cursor_position = 0;

    let result = update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::ReviseCommit),
    );

    assert_eq!(result, None);
    assert!(model.popup.is_none(), "No popup expected — using log view");
    assert!(
        matches!(model.view_mode, ViewMode::Log(LogType::Current, true)),
        "Expected log pick view"
    );
    assert_eq!(model.log_pick_on_select, Some(OnSelect::ReviseCommit));
}

#[test]
fn test_show_revise_commit_no_staged_changes_still_shows_log_pick() {
    // Revise does NOT require staged changes — should proceed without warning
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    // No staged changes at this point
    let mut model = create_model_from_test_repo(&test_repo);
    // cursor on a non-commit line
    model.ui_model.lines = vec![Line {
        content: LineContent::EmptyLine,
        section: None,
    }];
    model.ui_model.cursor_position = 0;

    let result = update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::ReviseCommit),
    );

    // Should NOT show a toast or dismiss — should show the log pick view
    assert_eq!(result, None);
    assert!(model.popup.is_none());
    assert!(
        matches!(model.view_mode, ViewMode::Log(LogType::Current, true)),
        "Expected log pick view even without staged changes"
    );
}

#[test]
fn test_revise_commit_select_confirm_routes_to_revise() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let commits = get_log_entries_for_test(&test_repo);
    let commit_hash = commits[0].hash.as_ref().unwrap().to_string();

    let mut model = create_model_from_test_repo(&test_repo);
    // Simulate log pick mode with the cursor on a commit
    model.view_mode = ViewMode::Log(LogType::Current, true);
    model.ui_model.lines = vec![make_log_line(&commit_hash, "First commit")];
    model.ui_model.cursor_position = 0;
    model.log_pick_on_select = Some(OnSelect::ReviseCommit);

    let result = update(
        &mut model,
        Message::Select(magi::msg::SelectMessage::Confirm),
    );

    assert_eq!(result, Some(Message::ReviseCommit(commit_hash)));
    assert_eq!(model.view_mode, ViewMode::Status);
    assert!(model.log_pick_on_select.is_none());
}
