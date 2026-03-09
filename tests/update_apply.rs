use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use magi::{
    git::{cherry_pick::cherry_pick_in_progress, log::get_log_entries, test_repo::TestRepo},
    keys::handle_key,
    model::{
        Line, LineContent, ViewMode,
        popup::{ApplyPopupState, PopupContent, PopupContentCommand},
    },
    msg::{ApplyCommand, LogType, Message, update::update},
};
use std::fs;

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

fn shift_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::SHIFT,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

// ── ShowApplyPopup — key binding ───────────────────────────────────────────────

#[test]
fn test_shift_a_key_shows_apply_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let model = create_model_from_test_repo(&test_repo);

    // 'A' is reported as Shift+a by crossterm
    let result = handle_key(shift_key(KeyCode::Char('A')), &model);
    assert_eq!(result, Some(Message::ShowApplyPopup));
}

// ── ShowApplyPopup — cursor on commit ─────────────────────────────────────────

#[test]
fn test_show_apply_popup_on_commit_line_selects_hash() {
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

    let result = update(&mut model, Message::ShowApplyPopup);

    assert_eq!(result, None);
    if let Some(PopupContent::Command(PopupContentCommand::Apply(state))) = &model.popup {
        assert!(!state.in_progress);
        assert_eq!(state.selected_commits, vec![expected_hash]);
    } else {
        panic!("Expected Apply popup");
    }
}

// ── ShowApplyPopup — cursor NOT on commit ─────────────────────────────────────

#[test]
fn test_show_apply_popup_on_non_commit_line_has_empty_commits() {
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

    update(&mut model, Message::ShowApplyPopup);

    if let Some(PopupContent::Command(PopupContentCommand::Apply(state))) = &model.popup {
        assert!(!state.in_progress);
        assert!(state.selected_commits.is_empty());
    } else {
        panic!("Expected Apply popup");
    }
}

// ── ShowApplyPopup — visual selection ─────────────────────────────────────────

#[test]
fn test_show_apply_popup_visual_selection_all_commits_collects_all_hashes() {
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

    model.ui_model.visual_mode_anchor = Some(first);
    model.ui_model.cursor_position = last;

    update(&mut model, Message::ShowApplyPopup);

    if let Some(PopupContent::Command(PopupContentCommand::Apply(state))) = &model.popup {
        assert!(!state.in_progress);
        assert_eq!(state.selected_commits.len(), commit_positions.len());
    } else {
        panic!("Expected Apply popup");
    }
}

// ── ShowApplyPopup — log view (LogLine) ───────────────────────────────────────

#[test]
fn test_show_apply_popup_on_log_line_selects_hash() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let log_lines: Vec<Line> = get_log_entries(&repo, LogType::Current)
        .unwrap()
        .into_iter()
        .map(|entry| Line {
            content: LineContent::LogLine(entry),
            section: None,
        })
        .collect();

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
    model.view_mode = ViewMode::Log(LogType::Current, false);
    model.ui_model.cursor_position = log_commit_pos;

    update(&mut model, Message::ShowApplyPopup);

    if let Some(PopupContent::Command(PopupContentCommand::Apply(state))) = &model.popup {
        assert!(!state.in_progress);
        assert_eq!(state.selected_commits, vec![expected_hash]);
    } else {
        panic!("Expected Apply popup with selected commit from log view");
    }
}

// ── Apply popup keys ──────────────────────────────────────────────────────────

#[test]
fn test_shift_a_in_apply_popup_with_commits_triggers_pick() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: false,
            selected_commits: vec!["abc1234".to_string()],
        },
    )));

    let result = handle_key(shift_key(KeyCode::Char('A')), &model);
    assert_eq!(
        result,
        Some(Message::Apply(ApplyCommand::Pick(vec![
            "abc1234".to_string()
        ])))
    );
}

#[test]
fn test_shift_a_in_apply_popup_without_commits_does_nothing() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: false,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(shift_key(KeyCode::Char('A')), &model);
    assert_eq!(result, None);
}

#[test]
fn test_shift_a_in_apply_popup_in_progress_triggers_continue() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: true,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(shift_key(KeyCode::Char('A')), &model);
    assert_eq!(result, Some(Message::Apply(ApplyCommand::Continue)));
}

#[test]
fn test_s_in_apply_popup_in_progress_triggers_skip() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: true,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Char('s')), &model);
    assert_eq!(result, Some(Message::Apply(ApplyCommand::Skip)));
}

#[test]
fn test_a_in_apply_popup_in_progress_triggers_abort() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: true,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Char('a')), &model);
    assert_eq!(result, Some(Message::Apply(ApplyCommand::Abort)));
}

#[test]
fn test_q_dismisses_apply_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: false,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Char('q')), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

#[test]
fn test_esc_dismisses_apply_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: false,
            selected_commits: vec![],
        },
    )));

    let result = handle_key(key(KeyCode::Esc), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

// ── cherry_pick_in_progress ───────────────────────────────────────────────────

#[test]
fn test_cherry_pick_in_progress_returns_true_with_cherry_pick_head() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let repo_path = test_repo.repo_path().to_path_buf();
    let git_dir = repo_path.join(".git");

    // Simulate a stopped cherry-pick
    fs::write(git_dir.join("CHERRY_PICK_HEAD"), "abc1234\n").unwrap();

    assert!(cherry_pick_in_progress(&repo_path));
}

#[test]
fn test_cherry_pick_in_progress_returns_false_without_marker() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let repo_path = test_repo.repo_path().to_path_buf();

    assert!(!cherry_pick_in_progress(&repo_path));
}

// ── ShowApplyPopup — in_progress when CHERRY_PICK_HEAD exists ─────────────────

#[test]
fn test_show_apply_popup_in_progress_when_cherry_pick_head_exists() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let git_dir = model.workdir.join(".git");
    fs::write(git_dir.join("CHERRY_PICK_HEAD"), "abc1234\n").unwrap();

    update(&mut model, Message::ShowApplyPopup);

    if let Some(PopupContent::Command(PopupContentCommand::Apply(state))) = &model.popup {
        assert!(state.in_progress);
        assert!(state.selected_commits.is_empty());
    } else {
        panic!("Expected Apply popup");
    }
}
