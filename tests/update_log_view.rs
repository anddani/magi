use magi::{
    git::test_repo::TestRepo,
    model::{
        LineContent, ViewMode,
        arguments::{Arguments, DEFAULT_LOG_LIMIT, LogArgument},
        popup::{InputContext, InputPopupState, PopupContent, PopupContentCommand},
    },
    msg::{CommitSelect, FixupType, InputMessage, LogType, Message, SelectMessage, update::update},
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
    assert!(matches!(
        model.view_mode,
        ViewMode::Log { picking: false, .. }
    ));
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
    assert!(matches!(
        model.view_mode,
        ViewMode::Log { picking: true, .. }
    ));
    assert_eq!(model.ui_model.cursor_position, 0);
    assert_eq!(model.ui_model.scroll_offset, 0);

    // Confirm the selection — back to status with the previous UI state
    update(&mut model, Message::Select(SelectMessage::Confirm));
    assert_eq!(model.view_mode, ViewMode::Status);
    assert_eq!(model.ui_model.cursor_position, 3);
    assert_eq!(model.ui_model.scroll_offset, 2);
}

// ── Show signatures argument (=S, --show-signature) ───────────────────────────

#[test]
fn test_show_log_with_show_signature_argument() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = Some(Arguments::log_args(
        [
            LogArgument::Graph,
            LogArgument::Decorate,
            LogArgument::ShowSignature,
        ]
        .into_iter()
        .collect(),
    ));

    update(&mut model, Message::ShowLog(LogType::Current));

    assert!(matches!(
        model.view_mode,
        ViewMode::Log {
            show_signature: true,
            ..
        }
    ));
    // The test commits are unsigned, so every commit reports 'N'
    let signatures: Vec<Option<char>> = model
        .ui_model
        .lines
        .iter()
        .filter_map(|line| match &line.content {
            LineContent::LogLine(entry) if entry.is_commit() => Some(entry.signature),
            _ => None,
        })
        .collect();
    assert!(!signatures.is_empty());
    assert!(signatures.iter().all(|s| *s == Some('N')));
}

// ── Show header argument (-h, ++header) ───────────────────────────────────────

#[test]
fn test_show_log_with_show_header_argument() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = Some(Arguments::log_args(
        [
            LogArgument::Graph,
            LogArgument::Decorate,
            LogArgument::ShowHeader,
        ]
        .into_iter()
        .collect(),
    ));

    update(&mut model, Message::ShowLog(LogType::Current));

    assert!(matches!(
        model.view_mode,
        ViewMode::Log {
            show_header: true,
            ..
        }
    ));
    // Every commit is followed by Author and Committer header lines
    let headers: Vec<String> = model
        .ui_model
        .lines
        .iter()
        .filter_map(|line| match &line.content {
            LineContent::LogLine(entry) if !entry.is_commit() => entry.message.clone(),
            _ => None,
        })
        .collect();
    assert!(headers.iter().any(|h| h.starts_with("Author:    ")));
    assert!(headers.iter().any(|h| h.starts_with("Committer: ")));
}

#[test]
fn test_show_log_without_show_header_argument() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    update(&mut model, Message::ShowLog(LogType::Current));

    assert!(matches!(
        model.view_mode,
        ViewMode::Log {
            show_header: false,
            ..
        }
    ));
    // Without ++header every log line is a commit or a graph-only line
    assert!(model.ui_model.lines.iter().all(|line| match &line.content {
        LineContent::LogLine(entry) => entry.is_commit() || entry.message.is_none(),
        _ => true,
    }));
}

#[test]
fn test_show_log_reflog_ignores_show_header_argument() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = Some(Arguments::log_args(
        [
            LogArgument::Graph,
            LogArgument::Decorate,
            LogArgument::ShowHeader,
        ]
        .into_iter()
        .collect(),
    ));

    update(&mut model, Message::ShowLog(LogType::Reflog));

    // Like Magit, reflogs never show headers
    assert!(matches!(
        model.view_mode,
        ViewMode::Log {
            show_header: false,
            ..
        }
    ));
    assert!(model.ui_model.lines.iter().all(|line| match &line.content {
        LineContent::LogLine(entry) => entry.is_commit(),
        _ => true,
    }));
}

#[test]
fn test_show_log_without_show_signature_argument() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    update(&mut model, Message::ShowLog(LogType::Current));

    assert!(matches!(
        model.view_mode,
        ViewMode::Log {
            show_signature: false,
            ..
        }
    ));
    assert!(model.ui_model.lines.iter().all(|line| match &line.content {
        LineContent::LogLine(entry) => entry.signature.is_none(),
        _ => true,
    }));
}

// ── Patch argument (-p, --patch) ──────────────────────────────────────────────

#[test]
fn test_show_log_with_patch_argument() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = Some(Arguments::log_args(
        [
            LogArgument::Graph,
            LogArgument::Decorate,
            LogArgument::Patch,
        ]
        .into_iter()
        .collect(),
    ));

    update(&mut model, Message::ShowLog(LogType::Current));

    assert!(matches!(model.view_mode, ViewMode::Log { patch: true, .. }));
}

#[test]
fn test_show_log_without_patch_argument() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    update(&mut model, Message::ShowLog(LogType::Current));

    assert!(matches!(
        model.view_mode,
        ViewMode::Log { patch: false, .. }
    ));
}

// ── Stat argument (-s, --stat) ────────────────────────────────────────────────

#[test]
fn test_show_log_with_stat_argument() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = Some(Arguments::log_args(
        [LogArgument::Graph, LogArgument::Decorate, LogArgument::Stat]
            .into_iter()
            .collect(),
    ));

    update(&mut model, Message::ShowLog(LogType::Current));

    assert!(matches!(model.view_mode, ViewMode::Log { stat: true, .. }));

    // --stat emits a diffstat summary line listing the changed file
    assert!(model.ui_model.lines.iter().any(|line| {
        match &line.content {
            LineContent::LogLine(entry) => entry
                .hash
                .as_deref()
                .is_some_and(|text| text.contains("file1.txt")),
            _ => false,
        }
    }));
}

#[test]
fn test_show_log_without_stat_argument() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    update(&mut model, Message::ShowLog(LogType::Current));

    assert!(matches!(model.view_mode, ViewMode::Log { stat: false, .. }));
}

// ── Commit limiting argument (-n, -n<N>) ──────────────────────────────────────

/// Counts the log lines that represent an actual commit (have a hash).
fn commit_count(model: &magi::model::Model) -> usize {
    model
        .ui_model
        .lines
        .iter()
        .filter(|line| match &line.content {
            LineContent::LogLine(entry) => entry.hash.is_some(),
            _ => false,
        })
        .count()
}

#[test]
fn test_show_log_defaults_to_default_limit() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    update(&mut model, Message::ShowLog(LogType::Current));

    assert!(matches!(
        model.view_mode,
        ViewMode::Log {
            limit: DEFAULT_LOG_LIMIT,
            ..
        }
    ));
}

#[test]
fn test_show_log_respects_custom_limit() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("a.txt", "a", "First commit");
    test_repo.commit_file("b.txt", "b", "Second commit");
    test_repo.commit_file("c.txt", "c", "Third commit");

    let mut model = create_model_from_test_repo(&test_repo);
    // Graph off so each commit maps to exactly one log line
    let mut args = Arguments::log_args([LogArgument::Decorate].into_iter().collect());
    *args.log_limit_mut().unwrap() = 2;
    model.arguments = Some(args);

    update(&mut model, Message::ShowLog(LogType::Current));

    // The limit is remembered for refreshing...
    assert!(matches!(model.view_mode, ViewMode::Log { limit: 2, .. }));
    // ...and only two of the three commits are shown
    assert_eq!(commit_count(&model), 2);
}

#[test]
fn test_log_limit_input_confirm_updates_arguments_and_reopens_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = Some(Arguments::log_args(
        [LogArgument::Graph, LogArgument::Decorate]
            .into_iter()
            .collect(),
    ));
    // Simulate the input popup opened via `-n` with the user having typed "10"
    model.popup = Some(PopupContent::Input(InputPopupState::with_text(
        InputContext::LogLimit,
        "10",
    )));
    model.arg_mode = true;

    update(&mut model, Message::Input(InputMessage::Confirm));

    // The new limit is stored and the log popup is reopened
    assert_eq!(
        model.arguments.as_ref().and_then(|a| a.log_limit()),
        Some(10)
    );
    assert_eq!(
        model.popup,
        Some(PopupContent::Command(PopupContentCommand::Log))
    );
    assert!(!model.arg_mode);
}

#[test]
fn test_log_limit_input_confirm_ignores_invalid_input() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.arguments = Some(Arguments::log_args(
        [LogArgument::Graph, LogArgument::Decorate]
            .into_iter()
            .collect(),
    ));
    model.popup = Some(PopupContent::Input(InputPopupState::with_text(
        InputContext::LogLimit,
        "notanumber",
    )));

    update(&mut model, Message::Input(InputMessage::Confirm));

    // Invalid input keeps the current (default) limit and still reopens the popup
    assert_eq!(
        model.arguments.as_ref().and_then(|a| a.log_limit()),
        Some(DEFAULT_LOG_LIMIT)
    );
    assert_eq!(
        model.popup,
        Some(PopupContent::Command(PopupContentCommand::Log))
    );
}

#[test]
fn test_show_log_limit_input_prefills_current_limit() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let mut args = Arguments::log_args([LogArgument::Decorate].into_iter().collect());
    *args.log_limit_mut().unwrap() = 77;
    model.arguments = Some(args);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Log));

    update(&mut model, Message::ShowLogLimitInput);

    match &model.popup {
        Some(PopupContent::Input(state)) => {
            assert_eq!(state.context, InputContext::LogLimit);
            assert_eq!(state.input.as_str(), "77");
        }
        other => panic!("expected input popup, got {other:?}"),
    }
}
