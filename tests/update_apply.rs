use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use magi::{
    git::{cherry_pick::cherry_pick_in_progress, log::get_log_entries, test_repo::TestRepo},
    keys::handle_key,
    model::{
        Line, LineContent,
        popup::{ApplyPopupState, PopupContent, PopupContentCommand},
        select_popup::OnSelect,
    },
    msg::{ApplyCommand, LogType, Message, OptionsSource, ShowSelectPopupConfig, update::update},
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

    let result = handle_key(shift_key(KeyCode::Char('A')), &model);
    assert_eq!(result, Some(Message::ShowApplyPopup));
}

// ── ShowApplyPopup — normal (not in progress) ──────────────────────────────────

#[test]
fn test_show_apply_popup_sets_state_not_in_progress() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowApplyPopup);

    assert_eq!(result, None);
    if let Some(PopupContent::Command(PopupContentCommand::Apply(state))) = &model.popup {
        assert!(!state.in_progress);
    } else {
        panic!("Expected Apply popup");
    }
}

// ── ShowApplyPopup — cursor on commit ─────────────────────────────────────────

#[test]
fn test_show_apply_popup_cursor_on_commit_collects_hash() {
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
        .expect("Expected a Commit line in the status view");
    model.ui_model.cursor_position = commit_pos;

    let expected_hash = if let LineContent::Commit(info) = &model.ui_model.lines[commit_pos].content
    {
        info.hash.clone()
    } else {
        panic!("Expected Commit line");
    };

    update(&mut model, Message::ShowApplyPopup);

    if let Some(PopupContent::Command(PopupContentCommand::Apply(state))) = &model.popup {
        assert_eq!(state.selected_commits, vec![expected_hash]);
    } else {
        panic!("Expected Apply popup");
    }
}

// ── ShowApplyPopup — cursor NOT on commit ──────────────────────────────────────

#[test]
fn test_show_apply_popup_cursor_not_on_commit_gives_empty_selection() {
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
        .commit("First commit")
        .write_file_content("file2.txt", "content2")
        .stage_files(&["file2.txt"])
        .commit("Second commit")
        .write_file_content("file3.txt", "content3")
        .stage_files(&["file3.txt"])
        .commit("Third commit");

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
        "Need at least 2 commits for visual selection test"
    );

    let first = *commit_positions.first().unwrap();
    let last = *commit_positions.last().unwrap();

    model.ui_model.visual_mode_anchor = Some(first);
    model.ui_model.cursor_position = last;

    // Collect hashes in display order (newest-first) to compare against reversed result
    let display_order_hashes: Vec<String> = commit_positions
        .iter()
        .filter_map(|&i| {
            if let LineContent::Commit(info) = &model.ui_model.lines[i].content {
                Some(info.hash.clone())
            } else {
                None
            }
        })
        .collect();

    update(&mut model, Message::ShowApplyPopup);

    if let Some(PopupContent::Command(PopupContentCommand::Apply(state))) = &model.popup {
        assert_eq!(state.selected_commits.len(), commit_positions.len());
        // Commits must be oldest-first (reverse of display order) for cherry-pick
        let expected: Vec<String> = display_order_hashes.into_iter().rev().collect();
        assert_eq!(state.selected_commits, expected);
    } else {
        panic!("Expected Apply popup");
    }
}

// ── Apply popup keys — normal mode (no commits selected) ─────────────────────

#[test]
fn test_shift_a_in_apply_popup_no_commits_shows_select_popup() {
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
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Apply (cherry-pick)".to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::ApplyPick,
        }))
    );
}

// ── Apply popup keys — normal mode (commits selected) ─────────────────────────

#[test]
fn test_shift_a_in_apply_popup_with_commit_picks_directly() {
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
fn test_shift_a_in_apply_popup_with_multiple_commits_picks_all() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: false,
            selected_commits: vec!["abc1234".to_string(), "def5678".to_string()],
        },
    )));

    let result = handle_key(shift_key(KeyCode::Char('A')), &model);
    assert_eq!(
        result,
        Some(Message::Apply(ApplyCommand::Pick(vec![
            "abc1234".to_string(),
            "def5678".to_string(),
        ])))
    );
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

// ── Apply popup keys — in-progress mode ───────────────────────────────────────

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
    } else {
        panic!("Expected Apply popup");
    }
}

// ── ShowApplyPopup — log view (LogLine) ───────────────────────────────────────

#[test]
fn test_show_apply_popup_cursor_on_log_line_collects_hash() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Switch to log view
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
        .expect("Expected a log line with a hash");

    let expected_hash = if let LineContent::LogLine(entry) = &log_lines[log_commit_pos].content {
        entry.hash.clone().unwrap()
    } else {
        panic!("Expected LogLine");
    };

    model.ui_model.lines = log_lines;
    model.ui_model.cursor_position = log_commit_pos;

    update(&mut model, Message::ShowApplyPopup);

    if let Some(PopupContent::Command(PopupContentCommand::Apply(state))) = &model.popup {
        assert_eq!(state.selected_commits, vec![expected_hash]);
    } else {
        panic!("Expected Apply popup");
    }
}

// ── Apply popup keys — 'a' (apply --no-commit, no commits selected) ───────────

#[test]
fn test_a_in_apply_popup_no_commits_shows_apply_select_popup() {
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

    let result = handle_key(key(KeyCode::Char('a')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Apply without committing".to_string(),
            source: OptionsSource::AllRefs,
            on_select: OnSelect::ApplyApply,
        }))
    );
}

// ── Apply popup keys — 'a' (apply --no-commit, commits selected) ──────────────

#[test]
fn test_a_in_apply_popup_with_commit_applies_directly() {
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

    let result = handle_key(key(KeyCode::Char('a')), &model);
    assert_eq!(
        result,
        Some(Message::Apply(ApplyCommand::Apply(vec![
            "abc1234".to_string()
        ])))
    );
}

#[test]
fn test_a_in_apply_popup_with_multiple_commits_applies_all() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState {
            in_progress: false,
            selected_commits: vec!["abc1234".to_string(), "def5678".to_string()],
        },
    )));

    let result = handle_key(key(KeyCode::Char('a')), &model);
    assert_eq!(
        result,
        Some(Message::Apply(ApplyCommand::Apply(vec![
            "abc1234".to_string(),
            "def5678".to_string(),
        ])))
    );
}
