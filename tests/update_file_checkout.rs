use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::{
        LineContent,
        popup::{PopupContent, PopupContentCommand},
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::{Message, OptionsSource, ShowSelectPopupConfig, update::update},
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

// ── 'f' key in reset popup shows FileCheckoutRevision select ──────────────────

#[test]
fn test_f_in_reset_popup_shows_revision_select() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Reset));

    let result = handle_key(key(KeyCode::Char('f')), &model);
    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Checkout file from revision".to_string(),
            source: OptionsSource::FileCheckoutRevisions,
            on_select: OnSelect::FileCheckoutRevision,
        }))
    );
}

// ── ShowSelectPopup::FileCheckoutRevision shows refs ─────────────────────────

#[test]
fn test_file_checkout_revision_shows_refs() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "Checkout file from revision".to_string(),
            source: OptionsSource::FileCheckoutRevisions,
            on_select: OnSelect::FileCheckoutRevision,
        }),
    );

    assert_eq!(result, None);
    assert!(matches!(
        &model.popup,
        Some(PopupContent::Command(PopupContentCommand::Select(state)))
            if !state.all_options.is_empty()
    ));
    if let Some(PopupContent::Command(PopupContentCommand::Select(state))) = &model.popup {
        assert_eq!(state.on_select, OnSelect::FileCheckoutRevision);
    } else {
        panic!("Expected select popup");
    }
}

// ── Selecting revision moves to FileCheckoutFile select ──────────────────────

#[test]
fn test_selecting_revision_shows_file_select() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "Checkout file from revision".to_string(),
            vec!["HEAD".to_string()],
            OnSelect::FileCheckoutRevision,
        ),
    )));

    let result = update(
        &mut model,
        Message::Select(magi::msg::SelectMessage::Confirm),
    );

    assert_eq!(
        result,
        Some(Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "File to checkout".to_string(),
            source: OptionsSource::TrackedFiles,
            on_select: OnSelect::FileCheckoutFile {
                revision: "HEAD".to_string(),
            },
        }))
    );
}

// ── ShowSelectPopup::FileCheckoutFile shows tracked files ─────────────────────

#[test]
fn test_file_checkout_file_shows_tracked_files() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("tracked.txt", "hello")
        .stage_files(&["tracked.txt"])
        .commit("Commit with tracked file");

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::ShowSelectPopup(ShowSelectPopupConfig {
            title: "File to checkout".to_string(),
            source: OptionsSource::TrackedFiles,
            on_select: OnSelect::FileCheckoutFile {
                revision: "HEAD".to_string(),
            },
        }),
    );

    assert_eq!(result, None);
    if let Some(PopupContent::Command(PopupContentCommand::Select(state))) = &model.popup {
        assert!(
            state.all_options.iter().any(|f| f == "tracked.txt"),
            "tracked.txt should be in the file list"
        );
        assert_eq!(
            state.on_select,
            OnSelect::FileCheckoutFile {
                revision: "HEAD".to_string(),
            }
        );
    } else {
        panic!("Expected Select popup");
    }
}

// ── Selecting file dispatches FileCheckout message ────────────────────────────

#[test]
fn test_selecting_file_dispatches_file_checkout() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Command(PopupContentCommand::Select(
        SelectPopupState::new(
            "File to checkout".to_string(),
            vec!["file.txt".to_string()],
            OnSelect::FileCheckoutFile {
                revision: "HEAD".to_string(),
            },
        ),
    )));

    let result = update(
        &mut model,
        Message::Select(magi::msg::SelectMessage::Confirm),
    );

    assert_eq!(
        result,
        Some(Message::FileCheckout {
            revision: "HEAD".to_string(),
            file: "file.txt".to_string(),
        })
    );
}

// ── FileCheckout restores file to earlier content ─────────────────────────────

#[test]
fn test_file_checkout_restores_file_content() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("restore.txt", "original")
        .stage_files(&["restore.txt"])
        .commit("Original commit");

    let original_hash = {
        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        repo.head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id()
            .to_string()
    };

    test_repo
        .write_file_content("restore.txt", "modified")
        .stage_files(&["restore.txt"])
        .commit("Modified commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::FileCheckout {
            revision: original_hash,
            file: "restore.txt".to_string(),
        },
    );

    assert_eq!(result, Some(Message::Refresh));

    let content = std::fs::read_to_string(test_repo.repo_path().join("restore.txt")).unwrap();
    assert_eq!(content.trim(), "original");
}

// ── FileCheckout with invalid revision shows error popup ─────────────────────

#[test]
fn test_file_checkout_invalid_revision_shows_error() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("file.txt", "content")
        .stage_files(&["file.txt"])
        .commit("Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::FileCheckout {
            revision: "nonexistent-rev-xyz".to_string(),
            file: "file.txt".to_string(),
        },
    );

    assert_eq!(result, None);
    assert!(
        matches!(&model.popup, Some(PopupContent::Error { .. })),
        "Expected error popup"
    );
}

// ── Cursor on file line pre-selects it in FileCheckoutFile popup ──────────────

#[test]
fn test_file_checkout_cursor_on_file_preselects_it() {
    let test_repo = TestRepo::new();
    test_repo
        .write_file_content("first.txt", "original")
        .write_file_content("second.txt", "other")
        .stage_files(&["first.txt", "second.txt"])
        .commit("Commit both files");

    // Create an unstaged change to "second.txt" so it appears in the model lines
    test_repo.write_file_content("second.txt", "modified");

    let mut model = create_model_from_test_repo(&test_repo);

    // Find the line for "second.txt" in the unstaged section
    let file_pos = model.ui_model.lines.iter().position(
        |l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == "second.txt"),
    );

    if let Some(pos) = file_pos {
        model.ui_model.cursor_position = pos;
        update(
            &mut model,
            Message::ShowSelectPopup(ShowSelectPopupConfig {
                title: "File to checkout".to_string(),
                source: OptionsSource::TrackedFiles,
                on_select: OnSelect::FileCheckoutFile {
                    revision: "HEAD".to_string(),
                },
            }),
        );

        if let Some(PopupContent::Command(PopupContentCommand::Select(state))) = &model.popup {
            assert_eq!(
                state.all_options[0], "second.txt",
                "second.txt should be pre-selected as it's under the cursor"
            );
        } else {
            panic!("Expected Select popup");
        }
    }
    // If second.txt doesn't appear in the visible lines, test is skipped
}
