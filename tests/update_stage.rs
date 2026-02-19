use magi::git::test_repo::TestRepo;
use magi::model::{LineContent, Model, SectionType};
use magi::msg::Message;
use magi::msg::update::update;

mod utils;

use crate::utils::create_model_from_test_repo;

/// Find the line index for an UnstagedFile with the given path.
fn find_unstaged_file_line(model: &Model, path: &str) -> Option<usize> {
    model
        .ui_model
        .lines
        .iter()
        .position(|l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == path))
}

/// Find the line index for an UntrackedFile with the given path.
fn find_untracked_file_line(model: &Model, path: &str) -> Option<usize> {
    model
        .ui_model
        .lines
        .iter()
        .position(|l| matches!(&l.content, LineContent::UntrackedFile(p) if p == path))
}

#[test]
fn test_visual_stage_two_collapsed_unstaged_files() {
    // This is the bug case: visual select on one collapsed unstaged file,
    // move down to another collapsed unstaged file, and press 's'.
    // Both files should be staged.
    let file_a = "file_a.txt";
    let file_b = "file_b.txt";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_a)
        .create_file(file_b)
        .stage_files(&[file_a, file_b])
        .commit("Add files")
        .write_file_content(file_a, "modified a")
        .write_file_content(file_b, "modified b");

    let mut model = create_model_from_test_repo(&test_repo);

    // Both files are collapsed by default (UnstagedFile sections are default_collapsed)
    let pos_a = find_unstaged_file_line(&model, "file_a.txt")
        .expect("file_a.txt should be in unstaged changes");
    let pos_b = find_unstaged_file_line(&model, "file_b.txt")
        .expect("file_b.txt should be in unstaged changes");

    // Collapse both files (they should be collapsed by default, but ensure it)
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::UnstagedFile {
            path: file_a.to_string(),
        });
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::UnstagedFile {
            path: file_b.to_string(),
        });

    // Enter visual mode on file_a
    model.ui_model.cursor_position = pos_a;
    update(&mut model, Message::EnterVisualMode);

    // Move cursor to file_b (simulating j keypresses, but we just set cursor directly
    // since move_down skips hidden lines properly)
    model.ui_model.cursor_position = pos_b;

    // Stage selected (the visual selection spans both files)
    let follow_up = update(&mut model, Message::StageSelected);
    assert_eq!(follow_up, Some(Message::Refresh));

    // Visual mode should be cleared after staging
    assert!(!model.ui_model.is_visual_mode());

    // Refresh to see the new state
    update(&mut model, Message::Refresh);

    // Both files should now be staged (appear as StagedFile, not UnstagedFile)
    let has_unstaged_a = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == file_a));
    let has_unstaged_b = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == file_b));

    assert!(
        !has_unstaged_a,
        "file_a.txt should no longer be in unstaged changes"
    );
    assert!(
        !has_unstaged_b,
        "file_b.txt should no longer be in unstaged changes"
    );

    // Verify they are in staged changes
    let has_staged_a = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::StagedFile(fc) if fc.path == file_a));
    let has_staged_b = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::StagedFile(fc) if fc.path == file_b));

    assert!(has_staged_a, "file_a.txt should be in staged changes");
    assert!(has_staged_b, "file_b.txt should be in staged changes");
}

#[test]
fn test_visual_stage_two_untracked_files() {
    let file_a = "file_a.txt";
    let file_b = "file_b.txt";
    let test_repo = TestRepo::new();
    test_repo.create_file(file_a).create_file(file_b);

    let mut model = create_model_from_test_repo(&test_repo);

    let pos_1 =
        find_untracked_file_line(&model, file_a).expect("file_a.txt should be in untracked files");
    let pos_2 =
        find_untracked_file_line(&model, file_b).expect("file_b.txt should be in untracked files");

    // Enter visual mode on new1.txt, extend to new2.txt
    model.ui_model.cursor_position = pos_1;
    update(&mut model, Message::EnterVisualMode);
    model.ui_model.cursor_position = pos_2;

    // Stage selected
    let follow_up = update(&mut model, Message::StageSelected);
    assert_eq!(follow_up, Some(Message::Refresh));
    update(&mut model, Message::Refresh);

    // Both files should be staged
    let has_untracked_1 = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::UntrackedFile(p) if p == file_a));
    let has_untracked_2 = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::UntrackedFile(p) if p == file_b));

    assert!(
        !has_untracked_1,
        "file_a.txt should no longer be in untracked files"
    );
    assert!(
        !has_untracked_2,
        "file_b.txt should no longer be in untracked files"
    );
}

#[test]
fn test_visual_stage_single_unstaged_file() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_name)
        .stage_files(&[file_name])
        .commit("Add test.txt")
        .write_file_content(file_name, "modified content");

    let mut model = create_model_from_test_repo(&test_repo);

    let pos = find_unstaged_file_line(&model, "test.txt")
        .expect("test.txt should be in unstaged changes");

    // Enter visual mode on the file (single-line selection)
    model.ui_model.cursor_position = pos;
    update(&mut model, Message::EnterVisualMode);

    // Stage selected
    let follow_up = update(&mut model, Message::StageSelected);
    assert_eq!(follow_up, Some(Message::Refresh));
    update(&mut model, Message::Refresh);

    // File should be staged
    let has_unstaged = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == "test.txt"));
    let has_staged = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::StagedFile(fc) if fc.path == "test.txt"));

    assert!(!has_unstaged, "test.txt should not be in unstaged changes");
    assert!(has_staged, "test.txt should be in staged changes");
}

#[test]
fn test_visual_stage_exits_visual_mode() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_name)
        .stage_files(&[file_name])
        .commit("Add test.txt")
        .write_file_content(file_name, "modified content");

    let mut model = create_model_from_test_repo(&test_repo);

    let pos = find_unstaged_file_line(&model, "test.txt")
        .expect("test.txt should be in unstaged changes");

    model.ui_model.cursor_position = pos;
    update(&mut model, Message::EnterVisualMode);
    assert!(model.ui_model.is_visual_mode());

    update(&mut model, Message::StageSelected);

    // Visual mode should be cleared
    assert!(!model.ui_model.is_visual_mode());
    assert_eq!(model.ui_model.visual_mode_anchor, None);
}

#[test]
fn test_visual_stage_three_unstaged_files_only_stages_selected() {
    // Visual select file_a and file_b but NOT file_c.
    // Only file_a and file_b should be staged.
    let file_a = "file_a.txt";
    let file_b = "file_b.txt";
    let file_c = "file_c.txt";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_a)
        .create_file(file_b)
        .create_file(file_c)
        .stage_files(&[file_a, file_b, file_c])
        .commit("Add files")
        .write_file_content(file_a, "modified a")
        .write_file_content(file_b, "modified b")
        .write_file_content(file_c, "modified c");

    let mut model = create_model_from_test_repo(&test_repo);

    // Collapse all files
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::UnstagedFile {
            path: file_a.to_string(),
        });
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::UnstagedFile {
            path: file_b.to_string(),
        });
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::UnstagedFile {
            path: file_c.to_string(),
        });

    let pos_a = find_unstaged_file_line(&model, file_a).unwrap();
    let pos_b = find_unstaged_file_line(&model, file_b).unwrap();

    // Visual select only file_a and file_b
    model.ui_model.cursor_position = pos_a;
    update(&mut model, Message::EnterVisualMode);
    model.ui_model.cursor_position = pos_b;

    update(&mut model, Message::StageSelected);
    update(&mut model, Message::Refresh);

    // file_a and file_b should be staged
    let has_unstaged_a = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == file_a));
    let has_unstaged_b = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == file_b));
    assert!(!has_unstaged_a, "file_a.txt should be staged");
    assert!(!has_unstaged_b, "file_b.txt should be staged");

    // file_c should still be unstaged
    let has_unstaged_c = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == file_c));
    assert!(
        has_unstaged_c,
        "file_c.txt should still be in unstaged changes"
    );
}
