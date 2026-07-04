use crossterm::event::KeyCode;
use magi::{
    git::test_repo::TestRepo,
    keys::handle_key,
    model::{LineContent, Model, SectionType, popup::ConfirmAction},
    msg::{DiscardSource, DiscardTarget, Message, update::update},
};

mod utils;
use utils::{
    assert_no_popup, create_model_from_test_repo, expect_confirm_popup, expect_error_popup,
    find_line, find_staged_file_line, find_unstaged_file_line, find_untracked_file_line, key,
};

// ── Shared fixtures ───────────────────────────────────────────────────────────

/// Original content producing a single hunk when lines 2 and 3 are modified.
const ONE_HUNK_ORIGINAL: &str = "line 1\nline 2\nline 3\nline 4\nline 5\n";
/// Modified content: diff lines are `-line 2`, `-line 3`, `+MODIFIED 2`, `+MODIFIED 3`.
const ONE_HUNK_MODIFIED: &str = "line 1\nMODIFIED 2\nMODIFIED 3\nline 4\nline 5\n";

/// Original content producing two hunks when lines 2 and 11 are modified
/// (the changes are far enough apart that the context lines do not overlap).
const TWO_HUNK_ORIGINAL: &str = "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\n\
                                 line 7\nline 8\nline 9\nline 10\nline 11\nline 12\n";
const TWO_HUNK_MODIFIED: &str = "line 1\nCHANGED 2\nline 3\nline 4\nline 5\nline 6\n\
                                 line 7\nline 8\nline 9\nline 10\nCHANGED 11\nline 12\n";

/// Expand the (default-collapsed) unstaged file section so its hunk/diff lines
/// are visible for cursor positioning and visual selection.
fn expand_unstaged_file(model: &mut Model, path: &str) {
    model
        .ui_model
        .collapsed_sections
        .remove(&SectionType::UnstagedFile {
            path: path.to_string(),
        });
}

/// Expand the (default-collapsed) staged file section.
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

/// Assert the confirm popup carries a DiscardChanges action with the given target.
#[track_caller]
fn expect_discard_confirm(model: &Model, expected: DiscardTarget) {
    let state = expect_confirm_popup(model);
    assert_eq!(
        state.on_confirm,
        ConfirmAction::DiscardChanges(expected),
        "Confirm popup carries the wrong discard target"
    );
}

// ── 'x' key triggers DiscardSelected ──────────────────────────────────────────

#[test]
fn test_x_key_dispatches_discard_selected() {
    let test_repo = TestRepo::new();
    test_repo
        .commit_file("test.txt", "content", "Initial commit")
        .write_file_content("test.txt", "modified");

    let model = create_model_from_test_repo(&test_repo);

    let result = handle_key(key(KeyCode::Char('x')), &model);
    assert_eq!(result, Some(Message::DiscardSelected));
}

// ── DiscardSelected mapping: file lines ───────────────────────────────────────

#[test]
fn test_discard_selected_unstaged_file_maps_to_unstaged_files() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, "original", "Initial commit")
        .write_file_content(file_name, "modified");

    let mut model = create_model_from_test_repo(&test_repo);
    model.ui_model.cursor_position =
        find_unstaged_file_line(&model, file_name).expect("test.txt should be in unstaged changes");

    let result = update(&mut model, Message::DiscardSelected);

    assert_eq!(result, None);
    expect_discard_confirm(
        &model,
        DiscardTarget::Files {
            paths: vec![file_name.to_string()],
            source: DiscardSource::Unstaged,
        },
    );
    assert_eq!(
        expect_confirm_popup(&model).message,
        "Discard changes in test.txt?"
    );
}

#[test]
fn test_discard_selected_untracked_file_maps_to_untracked_source() {
    let file_name = "new.txt";
    let test_repo = TestRepo::new();
    test_repo.create_file(file_name);

    let mut model = create_model_from_test_repo(&test_repo);
    model.ui_model.cursor_position =
        find_untracked_file_line(&model, file_name).expect("new.txt should be in untracked files");

    let result = update(&mut model, Message::DiscardSelected);

    assert_eq!(result, None);
    expect_discard_confirm(
        &model,
        DiscardTarget::Files {
            paths: vec![file_name.to_string()],
            source: DiscardSource::Untracked,
        },
    );
    assert_eq!(
        expect_confirm_popup(&model).message,
        "Trash untracked file new.txt?"
    );
}

#[test]
fn test_discard_selected_staged_file_maps_to_staged_source() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, "original", "Initial commit")
        .write_file_content(file_name, "staged modification")
        .stage_files(&[file_name]);

    let mut model = create_model_from_test_repo(&test_repo);
    model.ui_model.cursor_position =
        find_staged_file_line(&model, file_name).expect("test.txt should be in staged changes");

    let result = update(&mut model, Message::DiscardSelected);

    assert_eq!(result, None);
    expect_discard_confirm(
        &model,
        DiscardTarget::Files {
            paths: vec![file_name.to_string()],
            source: DiscardSource::Staged,
        },
    );
    assert_eq!(
        expect_confirm_popup(&model).message,
        "Discard staged changes in test.txt?"
    );
}

// ── DiscardSelected mapping: hunk and diff lines ──────────────────────────────

#[test]
fn test_discard_selected_hunk_line_maps_to_hunk_with_correct_index() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, TWO_HUNK_ORIGINAL, "Initial commit")
        .write_file_content(file_name, TWO_HUNK_MODIFIED);

    let mut model = create_model_from_test_repo(&test_repo);
    expand_unstaged_file(&mut model, file_name);
    model.ui_model.cursor_position =
        find_hunk_line(&model, 1).expect("Second hunk header should be present");

    let result = update(&mut model, Message::DiscardSelected);

    assert_eq!(result, None);
    expect_discard_confirm(
        &model,
        DiscardTarget::Hunk {
            path: file_name.to_string(),
            hunk_index: 1,
            source: DiscardSource::Unstaged,
        },
    );
}

#[test]
fn test_discard_selected_diff_line_maps_to_containing_hunk() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, TWO_HUNK_ORIGINAL, "Initial commit")
        .write_file_content(file_name, TWO_HUNK_MODIFIED);

    let mut model = create_model_from_test_repo(&test_repo);
    expand_unstaged_file(&mut model, file_name);

    // The line right after the second hunk header is a diff line of that hunk
    let diff_line_pos =
        find_hunk_line(&model, 1).expect("Second hunk header should be present") + 1;
    assert!(
        matches!(
            model.ui_model.lines[diff_line_pos].content,
            LineContent::DiffLine(_)
        ),
        "Expected a diff line after the hunk header"
    );
    model.ui_model.cursor_position = diff_line_pos;

    let result = update(&mut model, Message::DiscardSelected);

    assert_eq!(result, None);
    expect_discard_confirm(
        &model,
        DiscardTarget::Hunk {
            path: file_name.to_string(),
            hunk_index: 1,
            source: DiscardSource::Unstaged,
        },
    );
}

#[test]
fn test_discard_selected_staged_hunk_line_maps_to_staged_hunk() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, ONE_HUNK_ORIGINAL, "Initial commit")
        .write_file_content(file_name, ONE_HUNK_MODIFIED)
        .stage_files(&[file_name]);

    let mut model = create_model_from_test_repo(&test_repo);
    expand_staged_file(&mut model, file_name);
    model.ui_model.cursor_position =
        find_hunk_line(&model, 0).expect("Staged hunk header should be present");

    let result = update(&mut model, Message::DiscardSelected);

    assert_eq!(result, None);
    expect_discard_confirm(
        &model,
        DiscardTarget::Hunk {
            path: file_name.to_string(),
            hunk_index: 0,
            source: DiscardSource::Staged,
        },
    );
}

// ── DiscardSelected mapping: visual mode ──────────────────────────────────────

#[test]
fn test_discard_selected_visual_files_maps_to_all_selected_files() {
    let file_a = "file_a.txt";
    let file_b = "file_b.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_a, "original a", "Add file_a")
        .commit_file(file_b, "original b", "Add file_b")
        .write_file_content(file_a, "modified a")
        .write_file_content(file_b, "modified b");

    let mut model = create_model_from_test_repo(&test_repo);
    let pos_a = find_unstaged_file_line(&model, file_a).expect("file_a should be unstaged");
    let pos_b = find_unstaged_file_line(&model, file_b).expect("file_b should be unstaged");

    model.ui_model.cursor_position = pos_a;
    update(&mut model, Message::EnterVisualMode);
    model.ui_model.cursor_position = pos_b;

    let result = update(&mut model, Message::DiscardSelected);

    assert_eq!(result, None);
    expect_discard_confirm(
        &model,
        DiscardTarget::Files {
            paths: vec![file_a.to_string(), file_b.to_string()],
            source: DiscardSource::Unstaged,
        },
    );
}

#[test]
fn test_discard_selected_visual_diff_lines_maps_to_lines() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, ONE_HUNK_ORIGINAL, "Initial commit")
        .write_file_content(file_name, ONE_HUNK_MODIFIED);

    let mut model = create_model_from_test_repo(&test_repo);
    expand_unstaged_file(&mut model, file_name);

    // Hunk diff lines (0-based within the hunk, after the header):
    //   0: " line 1", 1: "-line 2", 2: "-line 3",
    //   3: "+MODIFIED 2", 4: "+MODIFIED 3", 5: " line 4", 6: " line 5"
    let hunk_pos = find_hunk_line(&model, 0).expect("Hunk header should be present");

    // Visually select "-line 2", "-line 3" and "+MODIFIED 2"
    model.ui_model.cursor_position = hunk_pos + 2;
    update(&mut model, Message::EnterVisualMode);
    model.ui_model.cursor_position = hunk_pos + 4;

    let result = update(&mut model, Message::DiscardSelected);

    assert_eq!(result, None);
    expect_discard_confirm(
        &model,
        DiscardTarget::Lines {
            path: file_name.to_string(),
            hunk_index: 0,
            line_indices: vec![1, 2, 3],
            source: DiscardSource::Unstaged,
        },
    );
}

#[test]
fn test_discard_selected_visual_hunks_maps_to_hunks_in_reverse_order() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, TWO_HUNK_ORIGINAL, "Initial commit")
        .write_file_content(file_name, TWO_HUNK_MODIFIED);

    let mut model = create_model_from_test_repo(&test_repo);
    expand_unstaged_file(&mut model, file_name);

    let hunk_0_pos = find_hunk_line(&model, 0).expect("First hunk header should be present");
    let hunk_1_pos = find_hunk_line(&model, 1).expect("Second hunk header should be present");

    // Visually select from the first hunk header through the second hunk header
    model.ui_model.cursor_position = hunk_0_pos;
    update(&mut model, Message::EnterVisualMode);
    model.ui_model.cursor_position = hunk_1_pos;

    let result = update(&mut model, Message::DiscardSelected);

    assert_eq!(result, None);
    // Indices are reversed (highest first) so hunks can be applied without shifts
    expect_discard_confirm(
        &model,
        DiscardTarget::Hunks {
            path: file_name.to_string(),
            hunk_indices: vec![1, 0],
            source: DiscardSource::Unstaged,
        },
    );
}

// ── ConfirmDiscard end-to-end: unstaged ───────────────────────────────────────

#[test]
fn test_confirm_discard_unstaged_file_reverts_working_tree() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, "original content", "Initial commit")
        .write_file_content(file_name, "modified content");

    let mut model = create_model_from_test_repo(&test_repo);
    model.ui_model.visual_mode_anchor = Some(0);

    let result = update(
        &mut model,
        Message::ConfirmDiscard(DiscardTarget::Files {
            paths: vec![file_name.to_string()],
            source: DiscardSource::Unstaged,
        }),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert_no_popup(&model);
    assert_eq!(model.ui_model.visual_mode_anchor, None);

    let content = std::fs::read_to_string(test_repo.repo_path().join(file_name)).unwrap();
    assert_eq!(content, "original content", "Content should be reverted");
}

#[test]
fn test_confirm_discard_unstaged_hunk_reverts_only_that_hunk() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, TWO_HUNK_ORIGINAL, "Initial commit")
        .write_file_content(file_name, TWO_HUNK_MODIFIED);

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::ConfirmDiscard(DiscardTarget::Hunk {
            path: file_name.to_string(),
            hunk_index: 0,
            source: DiscardSource::Unstaged,
        }),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert_no_popup(&model);

    let content = std::fs::read_to_string(test_repo.repo_path().join(file_name)).unwrap();
    assert!(
        content.contains("line 2"),
        "First hunk should be reverted: {content}"
    );
    assert!(
        content.contains("CHANGED 11"),
        "Second hunk should be untouched: {content}"
    );
}

#[test]
fn test_confirm_discard_unstaged_hunks_reverts_all_selected_hunks() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, TWO_HUNK_ORIGINAL, "Initial commit")
        .write_file_content(file_name, TWO_HUNK_MODIFIED);

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::ConfirmDiscard(DiscardTarget::Hunks {
            path: file_name.to_string(),
            hunk_indices: vec![1, 0],
            source: DiscardSource::Unstaged,
        }),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert_no_popup(&model);

    let content = std::fs::read_to_string(test_repo.repo_path().join(file_name)).unwrap();
    assert_eq!(content, TWO_HUNK_ORIGINAL, "Both hunks should be reverted");
}

#[test]
fn test_confirm_discard_unstaged_lines_reverts_only_selected_lines() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, ONE_HUNK_ORIGINAL, "Initial commit")
        .write_file_content(file_name, ONE_HUNK_MODIFIED);

    let mut model = create_model_from_test_repo(&test_repo);
    // Discard "-line 2" (index 1) and "+MODIFIED 2" (index 3): reverts line 2 only
    let result = update(
        &mut model,
        Message::ConfirmDiscard(DiscardTarget::Lines {
            path: file_name.to_string(),
            hunk_index: 0,
            line_indices: vec![1, 3],
            source: DiscardSource::Unstaged,
        }),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert_no_popup(&model);

    let content = std::fs::read_to_string(test_repo.repo_path().join(file_name)).unwrap();
    assert_eq!(
        content, "line 1\nline 2\nMODIFIED 3\nline 4\nline 5\n",
        "Only line 2's change should be reverted"
    );
}

// ── ConfirmDiscard end-to-end: untracked ──────────────────────────────────────

#[test]
fn test_confirm_discard_untracked_file_deletes_it_from_disk() {
    let file_a = "trash_me.txt";
    let file_b = "keep_me.txt";
    let test_repo = TestRepo::new();
    test_repo.create_file(file_a).create_file(file_b);

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::ConfirmDiscard(DiscardTarget::Files {
            paths: vec![file_a.to_string()],
            source: DiscardSource::Untracked,
        }),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert_no_popup(&model);
    assert!(
        !test_repo.repo_path().join(file_a).exists(),
        "Discarded untracked file should be deleted"
    );
    assert!(
        test_repo.repo_path().join(file_b).exists(),
        "Other untracked file should be untouched"
    );
}

// ── ConfirmDiscard end-to-end: staged ─────────────────────────────────────────

#[test]
fn test_confirm_discard_staged_file_reverts_index_and_working_tree() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, "original content", "Initial commit")
        .write_file_content(file_name, "staged modification")
        .stage_files(&[file_name]);

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::ConfirmDiscard(DiscardTarget::Files {
            paths: vec![file_name.to_string()],
            source: DiscardSource::Staged,
        }),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert_no_popup(&model);

    let content = std::fs::read_to_string(test_repo.repo_path().join(file_name)).unwrap();
    assert_eq!(
        content, "original content",
        "Working tree should be reverted"
    );

    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        !statuses.iter().any(|s| s.status().is_index_modified()),
        "Index should no longer contain the staged change"
    );
}

#[test]
fn test_confirm_discard_staged_new_file_deletes_it() {
    let file_name = "new_staged.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file("base.txt", "base", "Initial commit")
        .write_file_content(file_name, "brand new")
        .stage_files(&[file_name]);

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::ConfirmDiscard(DiscardTarget::Files {
            paths: vec![file_name.to_string()],
            source: DiscardSource::Staged,
        }),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert_no_popup(&model);
    assert!(
        !test_repo.repo_path().join(file_name).exists(),
        "Staged new file should be deleted from disk"
    );
}

#[test]
fn test_confirm_discard_staged_hunk_reverts_index_and_working_tree() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, ONE_HUNK_ORIGINAL, "Initial commit")
        .write_file_content(file_name, ONE_HUNK_MODIFIED)
        .stage_files(&[file_name]);

    let mut model = create_model_from_test_repo(&test_repo);
    let result = update(
        &mut model,
        Message::ConfirmDiscard(DiscardTarget::Hunk {
            path: file_name.to_string(),
            hunk_index: 0,
            source: DiscardSource::Staged,
        }),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert_no_popup(&model);

    let content = std::fs::read_to_string(test_repo.repo_path().join(file_name)).unwrap();
    assert_eq!(
        content, ONE_HUNK_ORIGINAL,
        "Working tree should be reverted"
    );

    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        !statuses.iter().any(|s| s.status().is_index_modified()),
        "Index should no longer contain the staged hunk"
    );
}

#[test]
fn test_confirm_discard_staged_lines_reverts_only_selected_lines() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, ONE_HUNK_ORIGINAL, "Initial commit")
        .write_file_content(file_name, ONE_HUNK_MODIFIED)
        .stage_files(&[file_name]);

    let mut model = create_model_from_test_repo(&test_repo);
    // Discard staged "-line 2" (index 1) and "+MODIFIED 2" (index 3)
    let result = update(
        &mut model,
        Message::ConfirmDiscard(DiscardTarget::Lines {
            path: file_name.to_string(),
            hunk_index: 0,
            line_indices: vec![1, 3],
            source: DiscardSource::Staged,
        }),
    );

    assert_eq!(result, Some(Message::Refresh));
    assert_no_popup(&model);

    let content = std::fs::read_to_string(test_repo.repo_path().join(file_name)).unwrap();
    assert_eq!(
        content, "line 1\nline 2\nMODIFIED 3\nline 4\nline 5\n",
        "Only line 2's staged change should be reverted"
    );

    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_index_modified()),
        "The MODIFIED 3 change should still be staged"
    );
}

// ── Full flow: DiscardSelected → confirm popup → ConfirmDiscard ───────────────

#[test]
fn test_discard_flow_end_to_end_reverts_file() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, "original content", "Initial commit")
        .write_file_content(file_name, "modified content");

    let mut model = create_model_from_test_repo(&test_repo);
    model.ui_model.cursor_position =
        find_unstaged_file_line(&model, file_name).expect("test.txt should be in unstaged changes");

    // 'x' shows the confirmation popup
    update(&mut model, Message::DiscardSelected);
    let state = expect_confirm_popup(&model);
    let ConfirmAction::DiscardChanges(target) = state.on_confirm.clone() else {
        panic!(
            "Expected DiscardChanges action, got: {:?}",
            state.on_confirm
        );
    };

    // Confirming applies the discard
    let result = update(&mut model, Message::ConfirmDiscard(target));

    assert_eq!(result, Some(Message::Refresh));
    assert_no_popup(&model);

    let content = std::fs::read_to_string(test_repo.repo_path().join(file_name)).unwrap();
    assert_eq!(content, "original content", "Content should be reverted");
}

// ── Error path ────────────────────────────────────────────────────────────────

#[test]
fn test_confirm_discard_missing_file_shows_error_popup() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("test.txt", "content", "Initial commit");

    let mut model = create_model_from_test_repo(&test_repo);
    model.ui_model.visual_mode_anchor = Some(0);

    let result = update(
        &mut model,
        Message::ConfirmDiscard(DiscardTarget::Files {
            paths: vec!["does_not_exist.txt".to_string()],
            source: DiscardSource::Unstaged,
        }),
    );

    assert_eq!(result, Some(Message::Refresh));
    let message = expect_error_popup(&model);
    assert!(
        message.contains("Error discarding"),
        "Unexpected error message: {message}"
    );
    assert_eq!(
        model.ui_model.visual_mode_anchor, None,
        "Visual anchor should be cleared even on error"
    );
}
