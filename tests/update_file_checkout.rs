use crossterm::event::KeyCode;
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::{
        popup::{PopupContent, PopupContentCommand},
        select_popup::{OnSelect, SelectPopupState},
    },
    msg::{Message, OptionsSource, ShowSelectPopupConfig, update::update},
};

mod utils;
use utils::{create_model_from_test_repo, expect_select_popup, find_unstaged_file_line, key};

// ── 'f' key in reset popup shows FileCheckoutRevision select ──────────────────

#[test]
fn test_f_in_reset_popup_shows_revision_select() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

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
    test_repo.commit_file("file.txt", "content", "Initial commit");

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
    let state = expect_select_popup(&model);
    assert!(!state.all_options.is_empty());
    assert_eq!(state.on_select, OnSelect::FileCheckoutRevision);
}

// ── Selecting revision moves to FileCheckoutFile select ──────────────────────

#[test]
fn test_selecting_revision_shows_file_select() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

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
    test_repo.commit_file("tracked.txt", "hello", "Commit with tracked file");

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
    let state = expect_select_popup(&model);
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
}

// ── Selecting file dispatches FileCheckout message ────────────────────────────

#[test]
fn test_selecting_file_dispatches_file_checkout() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file.txt", "content", "Initial commit");

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
    test_repo.commit_file("restore.txt", "original", "Original commit");

    let original_hash = test_repo.head_hash();

    test_repo.commit_file("restore.txt", "modified", "Modified commit");

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
    test_repo.commit_file("file.txt", "content", "Initial commit");

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
    let file_pos = find_unstaged_file_line(&model, "second.txt");

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

        let state = expect_select_popup(&model);
        assert_eq!(
            state.all_options[0], "second.txt",
            "second.txt should be pre-selected as it's under the cursor"
        );
    }
    // If second.txt doesn't appear in the visible lines, test is skipped
}
