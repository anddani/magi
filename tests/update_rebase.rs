use magi::{
    git::{log::get_log_entries, test_repo::TestRepo},
    model::{
        LineContent,
        popup::{
            CommitSelectPopupState, ConfirmAction, PopupContent, PopupContentCommand,
            RebasePopupState, SelectContext,
        },
    },
    msg::{LogType, Message, SelectMessage, SelectPopup, update::update},
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
            RebasePopupState { branch }
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
        Message::ShowSelectPopup(SelectPopup::RebaseElsewhere),
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
        Message::ShowSelectPopup(SelectPopup::RebaseElsewhere),
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
fn test_rebase_elsewhere_not_on_commit_shows_commit_select_popup() {
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
        Message::ShowSelectPopup(SelectPopup::RebaseElsewhere),
    );

    assert_eq!(result, None);
    assert!(matches!(
        &model.popup,
        Some(PopupContent::Command(PopupContentCommand::CommitSelect(_)))
    ));
    assert_eq!(model.select_context, Some(SelectContext::RebaseElsewhere));
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
        Some(Message::ShowSelectPopup(SelectPopup::RebaseElsewhere))
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
    let state = CommitSelectPopupState::new("Rebase elsewhere".to_string(), commits);
    model.popup = Some(PopupContent::Command(PopupContentCommand::CommitSelect(
        state,
    )));
    model.select_context = Some(SelectContext::RebaseElsewhere);

    let result = update(&mut model, Message::Select(SelectMessage::Confirm));

    assert_eq!(
        result,
        Some(Message::RebaseElsewhere(expected_hash.clone()))
    );
    assert!(model.popup.is_none());
    assert!(model.select_context.is_none());
}
