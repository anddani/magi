use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::popup::{MergePopupState, PopupContent, PopupContentCommand},
    msg::{MergeCommand, Message, SelectPopup, update::update},
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

// ── ShowMergePopup — key binding ───────────────────────────────────────────────

#[test]
fn test_m_key_shows_merge_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let model = create_model_from_test_repo(&test_repo);

    let result = handle_key(key(KeyCode::Char('m')), &model);
    assert_eq!(result, Some(Message::ShowMergePopup));
}

// ── ShowMergePopup — state ─────────────────────────────────────────────────────

#[test]
fn test_show_merge_popup_sets_state() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);

    let result = update(&mut model, Message::ShowMergePopup);

    assert_eq!(result, None);
    if let Some(PopupContent::Command(PopupContentCommand::Merge(state))) = &model.popup {
        assert!(!state.in_progress);
    } else {
        panic!("Expected Merge popup");
    }
}

// ── Merge popup keys — normal mode ────────────────────────────────────────────

#[test]
fn test_m_in_merge_popup_shows_select() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(
        MergePopupState { in_progress: false },
    )));

    let result = handle_key(key(KeyCode::Char('m')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(SelectPopup::MergeElsewhere))
    );
}

#[test]
fn test_q_dismisses_merge_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(
        MergePopupState { in_progress: false },
    )));

    let result = handle_key(key(KeyCode::Char('q')), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

#[test]
fn test_esc_dismisses_merge_popup() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(
        MergePopupState { in_progress: false },
    )));

    let result = handle_key(key(KeyCode::Esc), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}

// ── Merge popup keys — in_progress mode ──────────────────────────────────────

#[test]
fn test_m_in_merge_popup_in_progress_continues() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(
        MergePopupState { in_progress: true },
    )));

    let result = handle_key(key(KeyCode::Char('m')), &model);
    assert_eq!(result, Some(Message::Merge(MergeCommand::Continue)));
}

#[test]
fn test_a_in_merge_popup_in_progress_aborts() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(
        MergePopupState { in_progress: true },
    )));

    let result = handle_key(key(KeyCode::Char('a')), &model);
    assert_eq!(result, Some(Message::Merge(MergeCommand::Abort)));
}

#[test]
fn test_q_dismisses_merge_popup_in_progress() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file1.txt", "content1")
        .stage_files(&["file1.txt"])
        .commit("First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(
        MergePopupState { in_progress: true },
    )));

    let result = handle_key(key(KeyCode::Char('q')), &model);
    assert_eq!(result, Some(Message::DismissPopup));
}
