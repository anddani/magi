use std::fs;

use crossterm::event::KeyCode;
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::{
        LineContent, Model, PreviewLineType, SectionType, ToastStyle, ViewMode,
        popup::{ConfirmAction, ConfirmPopupState, PopupContent},
    },
    msg::{Message, ReverseTarget, update::update},
};

mod utils;
use utils::{
    create_model_from_test_repo, expect_confirm_popup, find_line, find_staged_file_line,
    find_unstaged_file_line, find_untracked_file_line, key,
};

// ── Shared fixtures ───────────────────────────────────────────────────────────

const ONE_HUNK_ORIGINAL: &str = "line 1\nline 2\nline 3\nline 4\nline 5\n";
const ONE_HUNK_MODIFIED: &str = "line 1\nMODIFIED 2\nMODIFIED 3\nline 4\nline 5\n";

/// Original/modified content producing two hunks (changes far enough apart
/// that the context lines do not overlap).
const TWO_HUNK_ORIGINAL: &str = "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\n\
                                 line 7\nline 8\nline 9\nline 10\nline 11\nline 12\n";
const TWO_HUNK_MODIFIED: &str = "line 1\nCHANGED 2\nline 3\nline 4\nline 5\nline 6\n\
                                 line 7\nline 8\nline 9\nline 10\nCHANGED 11\nline 12\n";

/// Creates a repo with `file_name` committed as `original` and the `staged`
/// content staged, plus a model of its status view.
fn create_staged_change_model(file_name: &str, original: &str, staged: &str) -> (TestRepo, Model) {
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, original, "Initial commit")
        .write_file_content(file_name, staged)
        .stage_files(&[file_name]);
    let model = create_model_from_test_repo(&test_repo);
    (test_repo, model)
}

/// Expand the (default-collapsed) staged file section so its hunk/diff lines
/// are visible for cursor positioning and visual selection.
fn expand_staged_file(model: &mut Model, path: &str) {
    model
        .ui_model
        .collapsed_sections
        .remove(&SectionType::StagedFile {
            path: path.to_string(),
        });
}

/// Find the line index of the diff hunk header with the given hunk index.
fn find_hunk_line(model: &Model, hunk_index: usize) -> Option<usize> {
    find_line(
        model,
        |c| matches!(c, LineContent::DiffHunk(h) if h.hunk_index == hunk_index),
    )
}

/// Assert the confirm popup carries a Reverse action with the given target.
#[track_caller]
fn expect_reverse_confirm(model: &Model, expected: ReverseTarget) {
    let state = expect_confirm_popup(model);
    assert_eq!(
        state.on_confirm,
        ConfirmAction::Reverse(expected),
        "Confirm popup carries the wrong reverse target"
    );
}

#[track_caller]
fn expect_warning_toast(model: &Model, expected_message: &str) {
    let toast = model.toast.as_ref().expect("Expected a toast");
    assert_eq!(toast.style, ToastStyle::Warning);
    assert_eq!(toast.message, expected_message);
}

fn file_content(test_repo: &TestRepo, file: &str) -> String {
    fs::read_to_string(test_repo.repo_path().join(file)).unwrap()
}

// ── '-' key triggers ReverseSelected ──────────────────────────────────────────

#[test]
fn test_minus_key_dispatches_reverse_selected() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("test.txt", "content", "Initial commit");

    let model = create_model_from_test_repo(&test_repo);

    let result = handle_key(key(KeyCode::Char('-')), &model);
    assert_eq!(result, Some(Message::ReverseSelected));
}

#[test]
fn test_y_in_reverse_confirm_popup_dispatches_confirm_reverse() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("test.txt", "content", "Initial commit");

    let target = ReverseTarget::Files {
        paths: vec!["test.txt".to_string()],
    };
    let mut model = create_model_from_test_repo(&test_repo);
    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message: "Reverse staged changes in test.txt?".to_string(),
        on_confirm: ConfirmAction::Reverse(target.clone()),
    }));

    let result = handle_key(key(KeyCode::Char('y')), &model);
    assert_eq!(result, Some(Message::ConfirmReverse(target)));
}

// ── ReverseSelected — uncommitted changes are rejected ────────────────────────

#[test]
fn test_reverse_selected_unstaged_file_shows_warning() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, "original", "Initial commit")
        .write_file_content(file_name, "modified");

    let mut model = create_model_from_test_repo(&test_repo);
    model.ui_model.cursor_position =
        find_unstaged_file_line(&model, file_name).expect("test.txt should be in unstaged changes");

    let result = update(&mut model, Message::ReverseSelected);

    assert_eq!(result, None);
    assert!(model.popup.is_none(), "No confirm popup should be shown");
    expect_warning_toast(&model, "Cannot reverse uncommitted changes");
    // The unstaged change is untouched
    assert_eq!(file_content(&test_repo, file_name), "modified");
}

#[test]
fn test_reverse_selected_untracked_file_shows_warning() {
    let file_name = "new.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file("other.txt", "content", "Initial commit")
        .create_file(file_name);

    let mut model = create_model_from_test_repo(&test_repo);
    model.ui_model.cursor_position =
        find_untracked_file_line(&model, file_name).expect("new.txt should be untracked");

    let result = update(&mut model, Message::ReverseSelected);

    assert_eq!(result, None);
    assert!(model.popup.is_none(), "No confirm popup should be shown");
    expect_warning_toast(&model, "Cannot reverse uncommitted changes");
}

// ── ReverseSelected mapping: staged changes ───────────────────────────────────

#[test]
fn test_reverse_selected_staged_file_maps_to_files_target() {
    let file_name = "test.txt";
    let (_test_repo, mut model) =
        create_staged_change_model(file_name, ONE_HUNK_ORIGINAL, ONE_HUNK_MODIFIED);
    model.ui_model.cursor_position =
        find_staged_file_line(&model, file_name).expect("test.txt should be in staged changes");

    let result = update(&mut model, Message::ReverseSelected);

    assert_eq!(result, None);
    expect_reverse_confirm(
        &model,
        ReverseTarget::Files {
            paths: vec![file_name.to_string()],
        },
    );
    assert_eq!(
        expect_confirm_popup(&model).message,
        "Reverse staged changes in test.txt?"
    );
}

#[test]
fn test_reverse_selected_staged_hunk_maps_to_hunk_target() {
    let file_name = "test.txt";
    let (_test_repo, mut model) =
        create_staged_change_model(file_name, ONE_HUNK_ORIGINAL, ONE_HUNK_MODIFIED);
    expand_staged_file(&mut model, file_name);
    model.ui_model.cursor_position =
        find_hunk_line(&model, 0).expect("Expected a staged hunk header");

    let result = update(&mut model, Message::ReverseSelected);

    assert_eq!(result, None);
    expect_reverse_confirm(
        &model,
        ReverseTarget::Hunk {
            path: file_name.to_string(),
            hunk_index: 0,
        },
    );
    assert_eq!(
        expect_confirm_popup(&model).message,
        "Reverse staged hunk in test.txt?"
    );
}

#[test]
fn test_reverse_selected_visual_mode_staged_hunks_maps_to_hunks_target() {
    let file_name = "test.txt";
    let (_test_repo, mut model) =
        create_staged_change_model(file_name, TWO_HUNK_ORIGINAL, TWO_HUNK_MODIFIED);
    expand_staged_file(&mut model, file_name);

    let first_hunk = find_hunk_line(&model, 0).expect("Expected first staged hunk");
    let second_hunk = find_hunk_line(&model, 1).expect("Expected second staged hunk");
    model.ui_model.visual_mode_anchor = Some(first_hunk);
    model.ui_model.cursor_position = second_hunk;

    let result = update(&mut model, Message::ReverseSelected);

    assert_eq!(result, None);
    expect_reverse_confirm(
        &model,
        ReverseTarget::Hunks {
            path: file_name.to_string(),
            hunk_indices: vec![1, 0],
        },
    );
    assert_eq!(
        expect_confirm_popup(&model).message,
        "Reverse 2 staged hunks in test.txt?"
    );
}

// ── ConfirmReverse — execution ────────────────────────────────────────────────

#[test]
fn test_confirm_reverse_staged_file_reverses_worktree_keeps_index() {
    let file_name = "test.txt";
    let (test_repo, mut model) =
        create_staged_change_model(file_name, ONE_HUNK_ORIGINAL, ONE_HUNK_MODIFIED);

    let result = update(
        &mut model,
        Message::ConfirmReverse(ReverseTarget::Files {
            paths: vec![file_name.to_string()],
        }),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert!(model.popup.is_none(), "Popup should be dismissed");
    let toast = model.toast.expect("Expected a success toast");
    assert_eq!(toast.style, ToastStyle::Success);
    // Working tree back to the committed content
    assert_eq!(file_content(&test_repo, file_name), ONE_HUNK_ORIGINAL);
    // ...but the change is still staged
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_index_modified()),
        "Change should still be staged after reverse"
    );
}

#[test]
fn test_confirm_reverse_error_shows_error_popup() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo.commit_file(file_name, "content\n", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    // A patch that does not match the working tree cannot be reversed
    let bogus_patch = "\
diff --git a/test.txt b/test.txt
--- a/test.txt
+++ b/test.txt
@@ -1,1 +1,1 @@
-does not exist
+neither does this
";
    let result = update(
        &mut model,
        Message::ConfirmReverse(ReverseTarget::Patch {
            patch: bogus_patch.to_string(),
        }),
    );

    assert_eq!(result, Some(Message::Refresh));
    match &model.popup {
        Some(PopupContent::Error { message }) => {
            assert!(message.starts_with("Error reversing:"), "got: {}", message);
        }
        other => panic!("Expected error popup, got {:?}", other),
    }
}

// ── ReverseSelected — preview mode ────────────────────────────────────────────

/// Creates a repo with two commits and a model previewing the HEAD commit,
/// whose diff added "line added\n" to the file.
fn create_commit_preview_model(file_name: &str) -> (TestRepo, Model) {
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, "line one\nline two\n", "Initial commit")
        .commit_file(file_name, "line one\nline two\nline added\n", "Add a line");

    let mut model = create_model_from_test_repo(&test_repo);
    let commit_pos = find_line(&model, |c| matches!(c, LineContent::Commit(_)))
        .expect("Expected a commit line in the status view");
    model.ui_model.cursor_position = commit_pos;
    update(&mut model, Message::ShowPreview);
    assert_eq!(model.view_mode, ViewMode::Preview);

    (test_repo, model)
}

#[test]
fn test_reverse_selected_in_preview_shows_confirm_with_patch() {
    let file_name = "test.txt";
    let (_test_repo, mut model) = create_commit_preview_model(file_name);

    let hunk_pos = find_line(
        &model,
        |c| matches!(c, LineContent::PreviewLine { line_type, .. } if *line_type == PreviewLineType::HunkHeader),
    )
    .expect("Expected a hunk header in the preview");
    model.ui_model.cursor_position = hunk_pos;

    let result = update(&mut model, Message::ReverseSelected);

    assert_eq!(result, None);
    let state = expect_confirm_popup(&model);
    assert_eq!(state.message, "Reverse hunk in test.txt?");
    match &state.on_confirm {
        ConfirmAction::Reverse(ReverseTarget::Patch { patch }) => {
            assert!(patch.contains("+line added"));
        }
        other => panic!("Expected a Patch reverse target, got {:?}", other),
    }
}

#[test]
fn test_confirm_reverse_from_preview_undoes_commit_in_working_tree() {
    let file_name = "test.txt";
    let (test_repo, mut model) = create_commit_preview_model(file_name);

    let hunk_pos = find_line(
        &model,
        |c| matches!(c, LineContent::PreviewLine { line_type, .. } if *line_type == PreviewLineType::HunkHeader),
    )
    .expect("Expected a hunk header in the preview");
    model.ui_model.cursor_position = hunk_pos;

    update(&mut model, Message::ReverseSelected);
    let target = match expect_confirm_popup(&model).on_confirm.clone() {
        ConfirmAction::Reverse(target) => target,
        other => panic!("Expected a reverse action, got {:?}", other),
    };

    let result = update(&mut model, Message::ConfirmReverse(target));

    assert_eq!(result, Some(Message::Refresh));
    // The commit's addition is undone in the working tree
    assert_eq!(file_content(&test_repo, file_name), "line one\nline two\n");
    // Stay in preview mode after reversing
    assert_eq!(model.view_mode, ViewMode::Preview);
}

#[test]
fn test_reverse_selected_on_commit_metadata_does_nothing() {
    let file_name = "test.txt";
    let (_test_repo, mut model) = create_commit_preview_model(file_name);
    model.ui_model.cursor_position = 0; // "commit <hash>" metadata line

    let result = update(&mut model, Message::ReverseSelected);

    assert_eq!(result, None);
    assert!(model.popup.is_none(), "No confirm popup should be shown");
}
