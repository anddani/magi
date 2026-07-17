use std::fs;

use crossterm::event::KeyCode;
use magi::{
    keys::handle_key,
    model::{LineContent, PreviewLineType, ToastStyle, ViewMode},
    msg::{Message, update::update},
};

mod utils;
use utils::{create_model_from_test_repo, find_line, key};

use magi::git::test_repo::TestRepo;
use magi::model::Model;

// ── ApplySelected — key binding ────────────────────────────────────────────────

#[test]
fn test_a_key_triggers_apply_selected() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "content1", "First commit");

    let model = create_model_from_test_repo(&test_repo);

    let result = handle_key(key(KeyCode::Char('a')), &model);
    assert_eq!(result, Some(Message::ApplySelected));
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Creates a repo with a committed file, an extra stashed change, and a model
/// previewing that stash. The working tree is clean, so applying the stash's
/// diff modifies the file.
fn create_stash_preview_model(files: &[(&str, &str, &str)]) -> (TestRepo, Model) {
    let test_repo = TestRepo::new();
    for (file, content, _) in files {
        test_repo.commit_file(file, content, &format!("Add {}", file));
    }
    for (file, _, stashed_content) in files {
        test_repo.write_file_content(file, stashed_content);
    }
    test_repo.create_stash("wip");

    let mut model = create_model_from_test_repo(&test_repo);
    let stash_pos = find_line(&model, |c| matches!(c, LineContent::Stash(_)))
        .expect("Expected a Stash line in the status view");
    model.ui_model.cursor_position = stash_pos;
    update(&mut model, Message::ShowPreview);
    assert_eq!(model.view_mode, ViewMode::Preview);

    (test_repo, model)
}

/// Find the index of the first preview line with the given content.
fn find_preview_line(model: &Model, text: &str) -> usize {
    find_line(
        model,
        |c| matches!(c, LineContent::PreviewLine { content, .. } if content == text),
    )
    .unwrap_or_else(|| panic!("Expected preview line: {}", text))
}

fn file_content(test_repo: &TestRepo, file: &str) -> String {
    fs::read_to_string(test_repo.repo_path().join(file)).unwrap()
}

// ── ApplySelected — preview mode ───────────────────────────────────────────────

#[test]
fn test_apply_hunk_from_stash_preview_modifies_working_tree() {
    let (test_repo, mut model) =
        create_stash_preview_model(&[("file1.txt", "one\ntwo\n", "one\ntwo\nthree\n")]);

    let hunk_pos = find_line(
        &model,
        |c| matches!(c, LineContent::PreviewLine { line_type, .. } if *line_type == PreviewLineType::HunkHeader),
    )
    .expect("Expected a hunk header in the preview");
    model.ui_model.cursor_position = hunk_pos;

    let result = update(&mut model, Message::ApplySelected);

    assert_eq!(result, None);
    assert_eq!(file_content(&test_repo, "file1.txt"), "one\ntwo\nthree\n");
    let toast = model.toast.expect("Expected a success toast");
    assert_eq!(toast.style, ToastStyle::Success);
    // Stay in preview mode after applying
    assert_eq!(model.view_mode, ViewMode::Preview);
}

#[test]
fn test_apply_on_diff_line_applies_containing_hunk() {
    let (test_repo, mut model) =
        create_stash_preview_model(&[("file1.txt", "one\ntwo\n", "one\ntwo\nthree\n")]);

    model.ui_model.cursor_position = find_preview_line(&model, "+three");

    update(&mut model, Message::ApplySelected);

    assert_eq!(file_content(&test_repo, "file1.txt"), "one\ntwo\nthree\n");
}

#[test]
fn test_apply_file_header_applies_only_that_file() {
    let (test_repo, mut model) = create_stash_preview_model(&[
        ("file1.txt", "a\n", "a\nb\n"),
        ("file2.txt", "x\n", "x\ny\n"),
    ]);

    model.ui_model.cursor_position =
        find_preview_line(&model, "diff --git a/file2.txt b/file2.txt");

    update(&mut model, Message::ApplySelected);

    assert_eq!(file_content(&test_repo, "file1.txt"), "a\n");
    assert_eq!(file_content(&test_repo, "file2.txt"), "x\ny\n");
}

#[test]
fn test_apply_visual_selection_spanning_files_applies_both() {
    let (test_repo, mut model) = create_stash_preview_model(&[
        ("file1.txt", "a\n", "a\nb\n"),
        ("file2.txt", "x\n", "x\ny\n"),
    ]);

    model.ui_model.visual_mode_anchor = Some(find_preview_line(&model, "+b"));
    model.ui_model.cursor_position = find_preview_line(&model, "+y");

    update(&mut model, Message::ApplySelected);

    assert_eq!(file_content(&test_repo, "file1.txt"), "a\nb\n");
    assert_eq!(file_content(&test_repo, "file2.txt"), "x\ny\n");
    // Visual mode is exited after applying
    assert!(!model.ui_model.is_visual_mode());
}

#[test]
fn test_apply_on_commit_metadata_does_nothing() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "one\n", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    let commit_pos = find_line(&model, |c| matches!(c, LineContent::Commit(_)))
        .expect("Expected a Commit line in the status view");
    model.ui_model.cursor_position = commit_pos;
    update(&mut model, Message::ShowPreview);
    assert_eq!(model.view_mode, ViewMode::Preview);

    // Cursor on the first line of the preview (commit metadata, not diff content)
    model.ui_model.cursor_position = 0;

    let result = update(&mut model, Message::ApplySelected);

    assert_eq!(result, None);
    assert_eq!(file_content(&test_repo, "file1.txt"), "one\n");
    assert!(model.toast.is_none());
    assert!(model.popup.is_none());
}

#[test]
fn test_apply_conflicting_patch_shows_error_popup() {
    let (test_repo, mut model) =
        create_stash_preview_model(&[("file1.txt", "one\ntwo\n", "one\ntwo\nthree\n")]);

    // Make the working tree conflict with the stashed hunk
    test_repo.write_file_content("file1.txt", "completely different\n");

    let hunk_pos = find_line(
        &model,
        |c| matches!(c, LineContent::PreviewLine { line_type, .. } if *line_type == PreviewLineType::HunkHeader),
    )
    .unwrap();
    model.ui_model.cursor_position = hunk_pos;

    update(&mut model, Message::ApplySelected);

    assert!(matches!(
        model.popup,
        Some(magi::model::popup::PopupContent::Error { .. })
    ));
    assert!(model.toast.is_none());
}

// ── ApplySelected — status view ────────────────────────────────────────────────

#[test]
fn test_apply_in_status_view_on_unstaged_file_shows_warning() {
    let test_repo = TestRepo::new();
    test_repo
        .commit_file("file1.txt", "one\n", "First commit")
        .write_file_content("file1.txt", "one\ntwo\n");

    let mut model = create_model_from_test_repo(&test_repo);
    model.ui_model.cursor_position = utils::find_unstaged_file_line(&model, "file1.txt")
        .expect("Expected an unstaged file line");

    let result = update(&mut model, Message::ApplySelected);

    assert_eq!(result, None);
    let toast = model.toast.expect("Expected a warning toast");
    assert_eq!(toast.style, ToastStyle::Warning);
    assert_eq!(toast.message, "Change is already in the working tree");
    assert_eq!(file_content(&test_repo, "file1.txt"), "one\ntwo\n");
}

#[test]
fn test_apply_in_status_view_on_non_change_line_does_nothing() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("file1.txt", "one\n", "First commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.ui_model.cursor_position = 0;

    let result = update(&mut model, Message::ApplySelected);

    assert_eq!(result, None);
    assert!(model.toast.is_none());
    assert!(model.popup.is_none());
}
