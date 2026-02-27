use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use magi::{
    git::{log::get_log_entries, test_repo::TestRepo},
    keys::handle_key,
    model::{
        Line, LineContent, ViewMode,
        popup::{PopupContent, PopupContentCommand, RevertPopupState},
    },
    msg::{LogType, Message, RevertCommand, update::update},
};

mod utils;
use utils::create_model_from_test_repo;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

// ── ShowRevertPopup — key binding ──────────────────────────────────────────────

#[test]
fn test_underscore_key_shows_revert_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let model = create_model_from_test_repo(&test_repo);

    // '_' is Shift+hyphen in many terminals; crossterm reports it as Char('_')
    let result = handle_key(key(KeyCode::Char('_')), &model);
    assert_eq!(result, Some(Message::ShowRevertPopup));
}

// ── ShowRevertPopup — cursor on commit ────────────────────────────────────────

#[test]
fn test_show_revert_popup_on_commit_line_selects_hash() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let commit_pos = model
        .ui_model
        .lines
        .iter()
        .position(|l| matches!(&l.content, LineContent::Commit(_)))
        .expect("Expected a commit line");
    model.ui_model.cursor_position = commit_pos;

    let expected_hash = if let LineContent::Commit(info) = &model.ui_model.lines[commit_pos].content
    {
        info.hash.clone()
    } else {
        panic!("Not a commit line");
    };

    let result = update(&mut model, Message::ShowRevertPopup);

    assert_eq!(result, None);
    if let Some(PopupContent::Command(PopupContentCommand::Revert(state))) = &model.popup {
        assert!(!state.in_progress);
        assert_eq!(state.selected_commits, vec![expected_hash]);
    } else {
        panic!("Expected Revert popup");
    }
}

// ── ShowRevertPopup — cursor NOT on commit ────────────────────────────────────

#[test]
fn test_show_revert_popup_on_non_commit_line_has_empty_commits() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let non_commit_pos = model
        .ui_model
        .lines
        .iter()
        .position(|l| !matches!(&l.content, LineContent::Commit(_) | LineContent::LogLine(_)))
        .expect("Expected a non-commit line");
    model.ui_model.cursor_position = non_commit_pos;

    update(&mut model, Message::ShowRevertPopup);

    if let Some(PopupContent::Command(PopupContentCommand::Revert(state))) = &model.popup {
        assert!(!state.in_progress);
        assert!(state.selected_commits.is_empty());
    } else {
        panic!("Expected Revert popup");
    }
}

// ── ShowRevertPopup — visual selection ───────────────────────────────────────

#[test]
fn test_show_revert_popup_visual_selection_all_commits_collects_all_hashes() {
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

    // Collect commit line positions
    let commit_positions: Vec<usize> = model
        .ui_model
        .lines
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            if matches!(&l.content, LineContent::Commit(_)) {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    assert!(
        commit_positions.len() >= 2,
        "Need at least 2 commits for this test"
    );

    let first = *commit_positions.first().unwrap();
    let last = *commit_positions.last().unwrap();

    // Set up visual selection spanning exactly the commit lines
    model.ui_model.visual_mode_anchor = Some(first);
    model.ui_model.cursor_position = last;

    update(&mut model, Message::ShowRevertPopup);

    if let Some(PopupContent::Command(PopupContentCommand::Revert(state))) = &model.popup {
        assert!(!state.in_progress);
        assert_eq!(state.selected_commits.len(), commit_positions.len());
    } else {
        panic!("Expected Revert popup");
    }
}

#[test]
fn test_show_revert_popup_visual_selection_with_non_commit_gives_empty() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Set visual selection that spans from line 0 (likely a non-commit ref/header) to end
    model.ui_model.visual_mode_anchor = Some(0);
    model.ui_model.cursor_position = model.ui_model.lines.len().saturating_sub(1);

    update(&mut model, Message::ShowRevertPopup);

    if let Some(PopupContent::Command(PopupContentCommand::Revert(state))) = &model.popup {
        assert!(!state.in_progress);
        // Selection contains non-commit lines → empty
        assert!(state.selected_commits.is_empty());
    } else {
        panic!("Expected Revert popup");
    }
}

// ── ShowRevertPopup — log view (LogLine) ──────────────────────────────────────

#[test]
fn test_show_revert_popup_on_log_line_selects_hash() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Populate the model with log-view lines (as ShowLog does)
    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let log_lines: Vec<Line> = get_log_entries(&repo, LogType::Current)
        .unwrap()
        .into_iter()
        .map(|entry| Line {
            content: LineContent::LogLine(entry),
            section: None,
        })
        .collect();

    // Find a log line that has a hash
    let log_commit_pos = log_lines
        .iter()
        .position(|l| matches!(&l.content, LineContent::LogLine(e) if e.hash.is_some()))
        .expect("Expected at least one log line with a hash");

    let expected_hash = if let LineContent::LogLine(entry) = &log_lines[log_commit_pos].content {
        entry.hash.clone().unwrap()
    } else {
        panic!("Not a log line");
    };

    model.ui_model.lines = log_lines;
    model.view_mode = ViewMode::Log(LogType::Current);
    model.ui_model.cursor_position = log_commit_pos;

    update(&mut model, Message::ShowRevertPopup);

    if let Some(PopupContent::Command(PopupContentCommand::Revert(state))) = &model.popup {
        assert!(!state.in_progress);
        assert_eq!(state.selected_commits, vec![expected_hash]);
    } else {
        panic!("Expected Revert popup with selected commit from log view");
    }
}

// ── Revert popup keys ─────────────────────────────────────────────────────────

#[test]
fn test_underscore_in_revert_popup_with_commits_triggers_revert() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: false,
            selected_commits: vec!["abc1234".to_string()],
        },
    )));

    let result = handle_key(key(KeyCode::Char('_')), &model);
    assert_eq!(
        result,
        Some(Message::Revert(RevertCommand::Commits(vec![
            "abc1234".to_string()
        ])))
    );
}

#[test]
fn test_underscore_in_revert_popup_without_commits_does_nothing() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: false,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Char('_')), &model);
    assert_eq!(result, None);
}

#[test]
fn test_underscore_in_revert_popup_in_progress_triggers_continue() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: true,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Char('_')), &model);
    assert_eq!(result, Some(Message::Revert(RevertCommand::Continue)));
}

#[test]
fn test_s_in_revert_popup_in_progress_triggers_skip() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: true,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Char('s')), &model);
    assert_eq!(result, Some(Message::Revert(RevertCommand::Skip)));
}

#[test]
fn test_a_in_revert_popup_in_progress_triggers_abort() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: true,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Char('a')), &model);
    assert_eq!(result, Some(Message::Revert(RevertCommand::Abort)));
}

#[test]
fn test_s_in_revert_popup_not_in_progress_does_nothing() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: false,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Char('s')), &model);
    assert_eq!(result, None);
}

#[test]
fn test_q_dismisses_revert_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: false,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Char('q')), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

#[test]
fn test_esc_dismisses_revert_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: false,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Esc), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

// ── v key — revert no commit ──────────────────────────────────────────────────

#[test]
fn test_v_in_revert_popup_with_commits_triggers_no_commit_revert() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: false,
            selected_commits: vec!["abc1234".to_string()],
        },
    )));

    let result = handle_key(key(KeyCode::Char('v')), &model);
    assert_eq!(
        result,
        Some(Message::Revert(RevertCommand::NoCommit(vec![
            "abc1234".to_string()
        ])))
    );
}

#[test]
fn test_v_in_revert_popup_without_commits_does_nothing() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: false,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Char('v')), &model);
    assert_eq!(result, None);
}

#[test]
fn test_v_in_revert_popup_in_progress_does_nothing() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(
        RevertPopupState {
            in_progress: true,
            selected_commits: vec![],
        },
    )));

    // 'v' is not a recognised key in in-progress mode
    let result = handle_key(key(KeyCode::Char('v')), &model);
    assert_eq!(result, None);
}
