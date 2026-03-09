use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use magi::{
    git::{cherry_pick::cherry_pick_in_progress, test_repo::TestRepo},
    keys::handle_key,
    model::popup::{ApplyPopupState, PopupContent, PopupContentCommand},
    msg::{ApplyCommand, CommitSelect, Message, update::update},
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

// ── Apply popup keys — normal mode ────────────────────────────────────────────

#[test]
fn test_shift_a_in_apply_popup_triggers_apply_pick_log_select() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Apply(
        ApplyPopupState { in_progress: false },
    )));

    let result = handle_key(shift_key(KeyCode::Char('A')), &model);
    assert_eq!(
        result,
        Some(Message::ShowCommitSelect(CommitSelect::ApplyPick))
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
        ApplyPopupState { in_progress: false },
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
        ApplyPopupState { in_progress: false },
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
        ApplyPopupState { in_progress: true },
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
        ApplyPopupState { in_progress: true },
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
        ApplyPopupState { in_progress: true },
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
