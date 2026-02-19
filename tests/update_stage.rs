use magi::git::stage::stage_files;
use magi::git::test_repo::TestRepo;
use magi::model::{LineContent, Model, SectionType};
use magi::msg::Message;
use magi::msg::update::update;
use std::fs;

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
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create two modified tracked files (need to commit them first, then modify)
    fs::write(repo_path.join("file_a.txt"), "original a").unwrap();
    fs::write(repo_path.join("file_b.txt"), "original b").unwrap();
    stage_files(repo_path, &["file_a.txt", "file_b.txt"]).unwrap();

    // Commit so they become tracked
    let repo = &test_repo.repo;
    let mut index = repo.index().unwrap();
    let tree_id = index.write_tree().unwrap();
    let sig = git2::Signature::now("Test", "test@test.com").unwrap();
    let parent = repo.head().unwrap().peel_to_commit().unwrap();
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Add files",
        &repo.find_tree(tree_id).unwrap(),
        &[&parent],
    )
    .unwrap();

    // Now modify both files to create unstaged changes
    fs::write(repo_path.join("file_a.txt"), "modified a").unwrap();
    fs::write(repo_path.join("file_b.txt"), "modified b").unwrap();

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
            path: "file_a.txt".to_string(),
        });
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::UnstagedFile {
            path: "file_b.txt".to_string(),
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
    let has_unstaged_a =
        model.ui_model.lines.iter().any(
            |l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == "file_a.txt"),
        );
    let has_unstaged_b =
        model.ui_model.lines.iter().any(
            |l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == "file_b.txt"),
        );

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
        .any(|l| matches!(&l.content, LineContent::StagedFile(fc) if fc.path == "file_a.txt"));
    let has_staged_b = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::StagedFile(fc) if fc.path == "file_b.txt"));

    assert!(has_staged_a, "file_a.txt should be in staged changes");
    assert!(has_staged_b, "file_b.txt should be in staged changes");
}

#[test]
fn test_visual_stage_two_untracked_files() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create two untracked files
    fs::write(repo_path.join("new1.txt"), "new content 1").unwrap();
    fs::write(repo_path.join("new2.txt"), "new content 2").unwrap();

    let mut model = create_model_from_test_repo(&test_repo);

    let pos_1 = find_untracked_file_line(&model, "new1.txt")
        .expect("new1.txt should be in untracked files");
    let pos_2 = find_untracked_file_line(&model, "new2.txt")
        .expect("new2.txt should be in untracked files");

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
        .any(|l| matches!(&l.content, LineContent::UntrackedFile(p) if p == "new1.txt"));
    let has_untracked_2 = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::UntrackedFile(p) if p == "new2.txt"));

    assert!(
        !has_untracked_1,
        "new1.txt should no longer be in untracked files"
    );
    assert!(
        !has_untracked_2,
        "new2.txt should no longer be in untracked files"
    );
}

#[test]
fn test_visual_stage_single_unstaged_file() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Modify the tracked file
    fs::write(repo_path.join("test.txt"), "modified content").unwrap();

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
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    fs::write(repo_path.join("test.txt"), "modified content").unwrap();

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
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create and commit three files
    fs::write(repo_path.join("file_a.txt"), "original a").unwrap();
    fs::write(repo_path.join("file_b.txt"), "original b").unwrap();
    fs::write(repo_path.join("file_c.txt"), "original c").unwrap();
    stage_files(repo_path, &["file_a.txt", "file_b.txt", "file_c.txt"]).unwrap();

    let repo = &test_repo.repo;
    let mut index = repo.index().unwrap();
    let tree_id = index.write_tree().unwrap();
    let sig = git2::Signature::now("Test", "test@test.com").unwrap();
    let parent = repo.head().unwrap().peel_to_commit().unwrap();
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Add files",
        &repo.find_tree(tree_id).unwrap(),
        &[&parent],
    )
    .unwrap();

    // Modify all three
    fs::write(repo_path.join("file_a.txt"), "modified a").unwrap();
    fs::write(repo_path.join("file_b.txt"), "modified b").unwrap();
    fs::write(repo_path.join("file_c.txt"), "modified c").unwrap();

    let mut model = create_model_from_test_repo(&test_repo);

    // Collapse all files
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::UnstagedFile {
            path: "file_a.txt".to_string(),
        });
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::UnstagedFile {
            path: "file_b.txt".to_string(),
        });
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::UnstagedFile {
            path: "file_c.txt".to_string(),
        });

    let pos_a = find_unstaged_file_line(&model, "file_a.txt").unwrap();
    let pos_b = find_unstaged_file_line(&model, "file_b.txt").unwrap();

    // Visual select only file_a and file_b
    model.ui_model.cursor_position = pos_a;
    update(&mut model, Message::EnterVisualMode);
    model.ui_model.cursor_position = pos_b;

    update(&mut model, Message::StageSelected);
    update(&mut model, Message::Refresh);

    // file_a and file_b should be staged
    let has_unstaged_a =
        model.ui_model.lines.iter().any(
            |l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == "file_a.txt"),
        );
    let has_unstaged_b =
        model.ui_model.lines.iter().any(
            |l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == "file_b.txt"),
        );
    assert!(!has_unstaged_a, "file_a.txt should be staged");
    assert!(!has_unstaged_b, "file_b.txt should be staged");

    // file_c should still be unstaged
    let has_unstaged_c =
        model.ui_model.lines.iter().any(
            |l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == "file_c.txt"),
        );
    assert!(
        has_unstaged_c,
        "file_c.txt should still be in unstaged changes"
    );
}
