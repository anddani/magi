use magi::git::stage::stage_files;
use magi::git::test_repo::TestRepo;
use magi::model::{LineContent, Model, SectionType};
use magi::msg::Message;
use magi::msg::update::update;
use std::fs;

mod utils;

use crate::utils::create_model_from_test_repo;

/// Find the line index for a StagedFile with the given path.
fn find_staged_file_line(model: &Model, path: &str) -> Option<usize> {
    model
        .ui_model
        .lines
        .iter()
        .position(|l| matches!(&l.content, LineContent::StagedFile(fc) if fc.path == path))
}

#[test]
fn test_visual_unstage_two_staged_files() {
    // Visual select on two staged files and press 'u'.
    // Both files should be unstaged.
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Modify the already tracked test.txt and create another tracked file
    fs::write(repo_path.join("test.txt"), "modified test.txt").unwrap();

    // Create and commit a second file so we have two tracked files
    fs::write(repo_path.join("second.txt"), "original second").unwrap();
    stage_files(repo_path, &["second.txt"]).unwrap();

    // Commit the second file
    let repo = &test_repo.repo;
    let mut index = repo.index().unwrap();
    let tree_id = index.write_tree().unwrap();
    let sig = git2::Signature::now("Test", "test@test.com").unwrap();
    let parent = repo.head().unwrap().peel_to_commit().unwrap();
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Add second file",
        &repo.find_tree(tree_id).unwrap(),
        &[&parent],
    )
    .unwrap();

    // Now modify both tracked files and stage them
    fs::write(repo_path.join("test.txt"), "modified test").unwrap();
    fs::write(repo_path.join("second.txt"), "modified second").unwrap();
    stage_files(repo_path, &["test.txt", "second.txt"]).unwrap();

    let mut model = create_model_from_test_repo(&test_repo);

    // Collapse both files (default behavior)
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::StagedFile {
            path: "test.txt".to_string(),
        });
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::StagedFile {
            path: "second.txt".to_string(),
        });

    // Both files should now be in staged changes
    let pos_a = find_staged_file_line(&model, "second.txt")
        .expect("second.txt should be in staged changes");
    let pos_b =
        find_staged_file_line(&model, "test.txt").expect("test.txt should be in staged changes");

    // Ensure pos_a < pos_b for proper visual selection
    let (pos_start, pos_end) = if pos_a < pos_b {
        (pos_a, pos_b)
    } else {
        (pos_b, pos_a)
    };

    // Enter visual mode on first file
    model.ui_model.cursor_position = pos_start;
    update(&mut model, Message::EnterVisualMode);

    // Move cursor to second file
    model.ui_model.cursor_position = pos_end;

    // Unstage selected (the visual selection spans both files)
    let follow_up = update(&mut model, Message::UnstageSelected);
    assert_eq!(follow_up, Some(Message::Refresh));

    // Visual mode should be cleared after unstaging
    assert!(!model.ui_model.is_visual_mode());

    // Refresh to see the new state
    update(&mut model, Message::Refresh);

    // Both files should now be unstaged (appear as UnstagedFile, not StagedFile)
    let has_staged_test = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::StagedFile(fc) if fc.path == "test.txt"));
    let has_staged_second = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::StagedFile(fc) if fc.path == "second.txt"));

    assert!(
        !has_staged_test,
        "test.txt should no longer be in staged changes"
    );
    assert!(
        !has_staged_second,
        "second.txt should no longer be in staged changes"
    );

    // Verify test.txt is in unstaged changes (Modified files become unstaged)
    let has_unstaged_test = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == "test.txt"));

    // second.txt may become either UnstagedFile or UntrackedFile depending on its
    // status (New files become Untracked when unstaged)
    let has_second_somewhere = model.ui_model.lines.iter().any(|l| {
        matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == "second.txt")
            || matches!(&l.content, LineContent::UntrackedFile(p) if p == "second.txt")
    });

    assert!(has_unstaged_test, "test.txt should be in unstaged changes");
    assert!(
        has_second_somewhere,
        "second.txt should be unstaged (either as Unstaged or Untracked)"
    );
}

#[test]
fn test_visual_unstage_single_staged_file() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Modify the tracked file and stage it
    fs::write(repo_path.join("test.txt"), "modified content").unwrap();
    stage_files(repo_path, &["test.txt"]).unwrap();

    let mut model = create_model_from_test_repo(&test_repo);

    let pos =
        find_staged_file_line(&model, "test.txt").expect("test.txt should be in staged changes");

    // Enter visual mode on the file (single-line selection)
    model.ui_model.cursor_position = pos;
    update(&mut model, Message::EnterVisualMode);

    // Unstage selected
    let follow_up = update(&mut model, Message::UnstageSelected);
    assert_eq!(follow_up, Some(Message::Refresh));
    update(&mut model, Message::Refresh);

    // File should be unstaged
    let has_staged = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::StagedFile(fc) if fc.path == "test.txt"));
    let has_unstaged = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == "test.txt"));

    assert!(!has_staged, "test.txt should not be in staged changes");
    assert!(has_unstaged, "test.txt should be in unstaged changes");
}

#[test]
fn test_visual_unstage_exits_visual_mode() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    fs::write(repo_path.join("test.txt"), "modified content").unwrap();
    stage_files(repo_path, &["test.txt"]).unwrap();

    let mut model = create_model_from_test_repo(&test_repo);

    let pos =
        find_staged_file_line(&model, "test.txt").expect("test.txt should be in staged changes");

    model.ui_model.cursor_position = pos;
    update(&mut model, Message::EnterVisualMode);
    assert!(model.ui_model.is_visual_mode());

    update(&mut model, Message::UnstageSelected);

    // Visual mode should be cleared
    assert!(!model.ui_model.is_visual_mode());
    assert_eq!(model.ui_model.visual_mode_anchor, None);
}

#[test]
fn test_visual_unstage_three_staged_files_only_unstages_selected() {
    // Visual select file_a and file_b but NOT file_c.
    // Only file_a and file_b should be unstaged.
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

    // Modify all three and stage them
    fs::write(repo_path.join("file_a.txt"), "modified a").unwrap();
    fs::write(repo_path.join("file_b.txt"), "modified b").unwrap();
    fs::write(repo_path.join("file_c.txt"), "modified c").unwrap();
    stage_files(repo_path, &["file_a.txt", "file_b.txt", "file_c.txt"]).unwrap();

    let mut model = create_model_from_test_repo(&test_repo);

    // Collapse all files
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::StagedFile {
            path: "file_a.txt".to_string(),
        });
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::StagedFile {
            path: "file_b.txt".to_string(),
        });
    model
        .ui_model
        .collapsed_sections
        .insert(SectionType::StagedFile {
            path: "file_c.txt".to_string(),
        });

    let pos_a = find_staged_file_line(&model, "file_a.txt").unwrap();
    let pos_b = find_staged_file_line(&model, "file_b.txt").unwrap();

    // Visual select only file_a and file_b
    model.ui_model.cursor_position = pos_a;
    update(&mut model, Message::EnterVisualMode);
    model.ui_model.cursor_position = pos_b;

    update(&mut model, Message::UnstageSelected);
    update(&mut model, Message::Refresh);

    // file_a and file_b should be unstaged
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
    assert!(!has_staged_a, "file_a.txt should be unstaged");
    assert!(!has_staged_b, "file_b.txt should be unstaged");

    // file_c should still be staged
    let has_staged_c = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::StagedFile(fc) if fc.path == "file_c.txt"));
    assert!(has_staged_c, "file_c.txt should still be in staged changes");
}
