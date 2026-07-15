use magi::{
    git::test_repo::TestRepo,
    model::{Line, LineContent, ViewMode},
    msg::{LogType, Message, update::update},
};

mod utils;
use utils::{create_model_from_test_repo, cursor_to_commit, find_commit_line, find_line};

// ── ShowPreview on a Commit line ──────────────────────────────────────────────

#[test]
fn test_show_preview_on_commit_line_enters_preview_mode() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Find a Commit line and place the cursor on it
    let commit_pos = find_commit_line(&model).expect("No commit line found");
    model.ui_model.cursor_position = commit_pos;

    let result = update(&mut model, Message::ShowPreview);

    assert_eq!(result, None);
    assert_eq!(model.view_mode, ViewMode::Preview);
    assert!(
        model
            .ui_model
            .lines
            .iter()
            .any(|l| matches!(&l.content, LineContent::PreviewLine { .. })),
        "Expected PreviewLine entries"
    );
    assert_eq!(model.ui_model.cursor_position, 0);
    assert_eq!(
        model.preview_return_ui_model.unwrap().cursor_position,
        commit_pos
    );
}

// ── ShowPreview on a Stash line ───────────────────────────────────────────────

#[test]
fn test_show_preview_on_stash_line_enters_preview_mode() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "initial", "Initial commit");
    test_repo.write_file_content("file.txt", "changed");

    let workdir = test_repo.repo.workdir().unwrap();
    magi::git::git_cmd(workdir, &["stash", "push", "-m", "test stash"])
        .output()
        .unwrap();

    let mut model = create_model_from_test_repo(&test_repo);

    let stash_pos =
        find_line(&model, |c| matches!(c, LineContent::Stash(_))).expect("No stash line found");
    model.ui_model.cursor_position = stash_pos;

    let result = update(&mut model, Message::ShowPreview);

    assert_eq!(result, None);
    assert_eq!(model.view_mode, ViewMode::Preview);
    assert!(
        model
            .ui_model
            .lines
            .iter()
            .any(|l| matches!(&l.content, LineContent::PreviewLine { .. })),
        "Expected PreviewLine entries"
    );
}

// ── ShowPreview on a LogLine ──────────────────────────────────────────────────

#[test]
fn test_show_preview_on_log_line_enters_preview_mode() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Switch to log view
    update(&mut model, Message::ShowLog(LogType::Current));
    assert!(matches!(model.view_mode, ViewMode::Log { picking: false, .. }));

    // Find a log line with a hash
    let log_pos = find_line(
        &model,
        |c| matches!(c, LineContent::LogLine(e) if e.hash.is_some()),
    )
    .expect("No log line with hash found");
    model.ui_model.cursor_position = log_pos;

    let result = update(&mut model, Message::ShowPreview);

    assert_eq!(result, None);
    assert_eq!(model.view_mode, ViewMode::Preview);
    assert!(
        model
            .ui_model
            .lines
            .iter()
            .any(|l| matches!(&l.content, LineContent::PreviewLine { .. }))
    );
}

// ── ShowPreview on a LogLine without hash ─────────────────────────────────────

#[test]
fn test_show_preview_on_graph_only_log_line_is_noop() {
    use magi::model::log_view::LogEntry;

    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Insert a graph-only log line (hash = None)
    let graph_only_line = Line {
        content: LineContent::LogLine(LogEntry::graph_only("|".to_string())),
        section: None,
    };
    model.ui_model.lines = vec![graph_only_line];
    model.ui_model.cursor_position = 0;
    model.view_mode = ViewMode::Log { log_type: LogType::Current, picking: false, graph: true, color: false };

    let result = update(&mut model, Message::ShowPreview);

    // Should be a no-op: None returned and view mode unchanged
    assert_eq!(result, None);
    assert!(matches!(model.view_mode, ViewMode::Log { picking: false, .. }));
}

// ── ExitPreview returns to Status ─────────────────────────────────────────────

#[test]
fn test_exit_preview_returns_to_status() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Enter preview
    cursor_to_commit(&mut model);
    update(&mut model, Message::ShowPreview);
    assert_eq!(model.view_mode, ViewMode::Preview);

    // Exit preview — returns Refresh message
    let result = update(&mut model, Message::ExitPreview);
    assert_eq!(result, Some(Message::Refresh));
    assert_eq!(model.view_mode, ViewMode::Status);
}

// ── ExitPreview returns to Log ────────────────────────────────────────────────

#[test]
fn test_exit_preview_returns_to_log() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Enter log
    update(&mut model, Message::ShowLog(LogType::Current));
    assert!(matches!(model.view_mode, ViewMode::Log { picking: false, .. }));

    // Enter preview from log
    let log_pos = find_line(
        &model,
        |c| matches!(c, LineContent::LogLine(e) if e.hash.is_some()),
    )
    .expect("No log line with hash found");
    let saved_cursor = log_pos;
    model.ui_model.cursor_position = log_pos;
    update(&mut model, Message::ShowPreview);
    assert_eq!(model.view_mode, ViewMode::Preview);

    // Exit preview
    let result = update(&mut model, Message::ExitPreview);
    assert_eq!(result, Some(Message::Refresh));
    assert!(matches!(model.view_mode, ViewMode::Log { picking: false, .. }));
    // Cursor should be restored
    assert_eq!(model.ui_model.cursor_position, saved_cursor);
    // Lines should contain LogLines again
    assert!(
        model
            .ui_model
            .lines
            .iter()
            .any(|l| matches!(&l.content, LineContent::LogLine(_)))
    );
}
