use std::fs;

use magi::{
    git::{log::get_log_entries, rebase::rebase_in_progress, test_repo::TestRepo},
    model::{
        Line, LineContent, SectionType, ViewMode,
        popup::{ConfirmAction, PopupContent, PopupContentCommand, RebasePopupState},
        select_popup::OnSelect,
    },
    msg::{CommitSelect, LogType, Message, RebaseCommand, SelectMessage, update::update},
};

mod utils;
use utils::create_model_from_test_repo;

// ── ShowRebasePopup ────────────────────────────────────────────────────────────

#[test]
fn test_show_rebase_popup_sets_popup_with_branch_name() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowRebasePopup);

    assert_eq!(result, None);
    assert!(matches!(
        &model.popup,
        Some(PopupContent::Command(PopupContentCommand::Rebase(
            RebasePopupState { branch, .. }
        ))) if !branch.is_empty()
    ));
}

#[test]
fn test_show_rebase_popup_captures_current_branch() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let expected_branch = model.git_info.current_branch().unwrap_or_default();

    update(&mut model, Message::ShowRebasePopup);

    if let Some(PopupContent::Command(PopupContentCommand::Rebase(state))) = &model.popup {
        assert_eq!(state.branch, expected_branch);
    } else {
        panic!("Expected Rebase popup");
    }
}

// ── RebaseElsewhere - cursor on commit ────────────────────────────────────────

#[test]
fn test_rebase_elsewhere_on_commit_shows_confirmation() {
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

    // Place cursor on a commit line
    let commit_line = model
        .ui_model
        .lines
        .iter()
        .position(|l| matches!(&l.content, LineContent::Commit(_)))
        .expect("Expected a commit line in the model");

    model.ui_model.cursor_position = commit_line;

    let result = update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::RebaseElsewhere),
    );

    assert_eq!(result, None);
    assert!(matches!(
        &model.popup,
        Some(PopupContent::Confirm(state))
            if matches!(&state.on_confirm, ConfirmAction::RebaseElsewhere(_))
    ));
}

#[test]
fn test_rebase_elsewhere_confirmation_message_contains_hash() {
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

    let commit_line = model
        .ui_model
        .lines
        .iter()
        .position(|l| matches!(&l.content, LineContent::Commit(_)))
        .expect("Expected a commit line in the model");

    model.ui_model.cursor_position = commit_line;

    // Get the expected hash
    let expected_hash =
        if let LineContent::Commit(info) = &model.ui_model.lines[commit_line].content {
            info.hash.clone()
        } else {
            panic!("Not a commit line");
        };

    update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::RebaseElsewhere),
    );

    if let Some(PopupContent::Confirm(state)) = &model.popup {
        assert!(state.message.contains(&expected_hash));
        assert!(matches!(
            &state.on_confirm,
            ConfirmAction::RebaseElsewhere(hash) if *hash == expected_hash
        ));
    } else {
        panic!("Expected Confirm popup");
    }
}

// ── RebaseElsewhere - cursor NOT on commit ────────────────────────────────────

#[test]
fn test_rebase_elsewhere_not_on_commit_shows_log_pick_view() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Find a non-commit line (section header, empty line, etc.)
    let non_commit_pos = model
        .ui_model
        .lines
        .iter()
        .position(|l| !matches!(&l.content, LineContent::Commit(_) | LineContent::LogLine(_)))
        .expect("Expected at least one non-commit line");

    model.ui_model.cursor_position = non_commit_pos;

    let result = update(
        &mut model,
        Message::ShowCommitSelect(CommitSelect::RebaseElsewhere),
    );

    assert_eq!(result, None);
    assert!(model.popup.is_none(), "No popup expected — using log view");
    assert!(
        matches!(model.view_mode, ViewMode::Log(LogType::AllReferences, true)),
        "Expected AllReferences log pick view"
    );
    assert_eq!(model.log_pick_on_select, Some(OnSelect::RebaseElsewhere));
}

// ── Keys ──────────────────────────────────────────────────────────────────────

#[test]
fn test_r_key_shows_rebase_popup() {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use magi::keys::handle_key;

    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let model = create_model_from_test_repo(&test_repo);

    let key = KeyEvent {
        code: KeyCode::Char('r'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    let result = handle_key(key, &model);
    assert_eq!(result, Some(Message::ShowRebasePopup));
}

#[test]
fn test_e_in_rebase_popup_shows_rebase_elsewhere() {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use magi::keys::handle_key;

    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(
        RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
        },
    )));

    let key = KeyEvent {
        code: KeyCode::Char('e'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    let result = handle_key(key, &model);
    assert_eq!(
        result,
        Some(Message::ShowCommitSelect(CommitSelect::RebaseElsewhere))
    );
}

#[test]
fn test_esc_dismisses_rebase_popup() {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use magi::keys::handle_key;

    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(
        RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
        },
    )));

    let key = KeyEvent {
        code: KeyCode::Esc,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    let result = handle_key(key, &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

#[test]
fn test_q_dismisses_rebase_popup() {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use magi::keys::handle_key;

    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(
        RebasePopupState {
            branch: "main".to_string(),
            in_progress: false,
        },
    )));

    let key = KeyEvent {
        code: KeyCode::Char('q'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    let result = handle_key(key, &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

// ── Select confirm → RebaseElsewhere ─────────────────────────────────────────

#[test]
fn test_select_confirm_rebase_elsewhere_context_returns_rebase_message() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
    let mut commits = get_log_entries(&repo, LogType::Current).unwrap();
    commits.retain(|e| e.is_commit());

    let mut model = create_model_from_test_repo(&test_repo);

    let expected_hash = commits[0].hash.as_ref().unwrap().clone();

    // Set up model in log pick mode (new approach: no popup, log view)
    model.ui_model.lines = commits
        .into_iter()
        .map(|entry| Line {
            content: LineContent::LogLine(entry),
            section: None,
        })
        .collect();
    model.ui_model.cursor_position = 0;
    model.view_mode = ViewMode::Log(LogType::AllReferences, true);
    model.log_pick_on_select = Some(OnSelect::RebaseElsewhere);

    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::Rebase(RebaseCommand::Elsewhere(
            expected_hash.clone()
        )))
    );
    assert_eq!(model.view_mode, ViewMode::Status);
    assert!(model.log_pick_on_select.is_none());
}

// ── ShowRebasePopup — not in_progress (normal repo) ──────────────────────────

#[test]
fn test_show_rebase_popup_not_in_progress_for_clean_repo() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    update(&mut model, Message::ShowRebasePopup);

    if let Some(PopupContent::Command(PopupContentCommand::Rebase(state))) = &model.popup {
        assert!(
            !state.in_progress,
            "Expected in_progress = false for a clean repo"
        );
    } else {
        panic!("Expected Rebase popup");
    }
}

// ── ShowRebasePopup — in_progress when rebase-merge dir exists ────────────────

#[test]
fn test_show_rebase_popup_in_progress_when_rebase_merge_dir_exists() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Simulate a rebase in progress by creating the rebase-merge directory
    let git_dir = model.workdir.join(".git");
    let rebase_merge = git_dir.join("rebase-merge");
    fs::create_dir_all(&rebase_merge).unwrap();

    update(&mut model, Message::ShowRebasePopup);

    if let Some(PopupContent::Command(PopupContentCommand::Rebase(state))) = &model.popup {
        assert!(
            state.in_progress,
            "Expected in_progress = true when rebase-merge dir exists"
        );
    } else {
        panic!("Expected Rebase popup");
    }

    // Cleanup
    fs::remove_dir_all(&rebase_merge).unwrap();
}

// ── rebase_in_progress detection ─────────────────────────────────────────────

#[test]
fn test_rebase_in_progress_false_for_clean_repo() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let model = create_model_from_test_repo(&test_repo);
    assert!(!rebase_in_progress(&model.workdir));
}

// ── Keys in rebase popup when in_progress ─────────────────────────────────────

#[test]
fn test_r_key_in_in_progress_popup_continues_rebase() {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use magi::keys::handle_key;

    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(
        RebasePopupState {
            branch: "main".to_string(),
            in_progress: true,
        },
    )));

    let key = KeyEvent {
        code: KeyCode::Char('r'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    let result = handle_key(key, &model);
    assert_eq!(result, Some(Message::Rebase(RebaseCommand::Continue)));
}

#[test]
fn test_s_key_in_in_progress_popup_skips_rebase() {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use magi::keys::handle_key;

    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(
        RebasePopupState {
            branch: "main".to_string(),
            in_progress: true,
        },
    )));

    let key = KeyEvent {
        code: KeyCode::Char('s'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    let result = handle_key(key, &model);
    assert_eq!(result, Some(Message::Rebase(RebaseCommand::Skip)));
}

#[test]
fn test_a_key_in_in_progress_popup_aborts_rebase() {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use magi::keys::handle_key;

    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(
        RebasePopupState {
            branch: "main".to_string(),
            in_progress: true,
        },
    )));

    let key = KeyEvent {
        code: KeyCode::Char('a'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    let result = handle_key(key, &model);
    assert_eq!(result, Some(Message::Rebase(RebaseCommand::Abort)));
}

#[test]
fn test_e_key_has_no_effect_in_in_progress_popup() {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use magi::keys::handle_key;

    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Rebase(
        RebasePopupState {
            branch: "main".to_string(),
            in_progress: true,
        },
    )));

    let key = KeyEvent {
        code: KeyCode::Char('e'),
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    // 'e' should not trigger elsewhere in in_progress mode
    let result = handle_key(key, &model);
    assert_eq!(result, None);
}

// ── Rebasing section shown in status view when rebase in progress ─────────────

#[test]
fn test_rebasing_section_shown_when_rebase_merge_dir_and_stopped_sha_exist() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    // Simulate a stopped rebase
    let git_dir = model.workdir.join(".git");
    let rebase_merge = git_dir.join("rebase-merge");
    fs::create_dir_all(&rebase_merge).unwrap();
    fs::write(rebase_merge.join("stopped-sha"), "abc1234abcdef0000\n").unwrap();

    // Refresh to pick up the new state
    update(&mut model, Message::Refresh);

    let has_rebasing_section = model
        .ui_model
        .lines
        .iter()
        .any(|l| l.section == Some(SectionType::Rebasing));
    assert!(
        has_rebasing_section,
        "Expected a Rebasing section in the UI"
    );

    let has_rebasing_entry = model.ui_model.lines.iter().any(|l| {
        matches!(
            &l.content,
            LineContent::RebasingEntry {
                is_current: true,
                ..
            }
        )
    });
    assert!(
        has_rebasing_entry,
        "Expected a current rebasing entry in the UI"
    );

    // Cleanup
    fs::remove_dir_all(&rebase_merge).unwrap();
}
