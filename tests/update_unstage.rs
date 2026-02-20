use magi::git::stage::{stage_files, stage_hunk};
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
    let file_a = "file_a.txt";
    let file_b = "file_b.txt";

    // Visual select file_a and file_b but NOT file_c.
    // Only file_a and file_b should be unstaged.
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_a)
        .create_file(file_b)
        .stage_files(&[file_a, file_b])
        .commit("Commit 2 files")
        .write_file_content(file_a, "modified a")
        .write_file_content(file_b, "modified b")
        .stage_files(&[file_a, file_b]);

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
    let pos_a =
        find_staged_file_line(&model, file_a).expect("file_a.txt should be in staged changes");
    let pos_b =
        find_staged_file_line(&model, file_b).expect("file_b.txt should be in staged changes");

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
    let has_staged_file_a = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::StagedFile(fc) if fc.path == "file_a.txt"));
    let has_staged_file_b = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::StagedFile(fc) if fc.path == "file_b.txt"));

    assert!(
        !has_staged_file_a,
        "file_a.txt should no longer be in staged changes"
    );
    assert!(
        !has_staged_file_b,
        "file_b.txt should no longer be in staged changes"
    );

    // Verify file_a.txt is in unstaged changes (Modified files become unstaged)
    let has_unstaged_file_a = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == file_a));

    // file_b.txt may become either UnstagedFile or UntrackedFile depending on its
    // status (New files become Untracked when unstaged)
    let has_file_b_somewhere = model.ui_model.lines.iter().any(|l| {
        matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == file_b)
            || matches!(&l.content, LineContent::UntrackedFile(p) if p == file_b)
    });

    assert!(
        has_unstaged_file_a,
        "file_a.txt should be in unstaged changes"
    );
    assert!(
        has_file_b_somewhere,
        "file_b.txt should be unstaged (either as Unstaged or Untracked)"
    );
}

/// When a file has two hunks and one is staged, visually selecting
/// diff lines in the staged hunk and pressing 'u' must correctly unstage those lines.
#[test]
fn test_visual_unstage_lines_from_staged_hunk_when_other_hunk_is_unstaged() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();

    // Create a file with 20 lines (enough separation for two distinct hunks)
    let mut content = String::new();
    for i in 1..=20 {
        content.push_str(&format!("line {}\n", i));
    }
    test_repo
        .create_file(file_name)
        .write_file_content(file_name, &content)
        .stage_files(&[file_name])
        .commit("Initial 20 lines");

    // Modify lines 2 and 19 to create two separate hunks
    let modified = content
        .replace("line 2\n", "MODIFIED 2\n")
        .replace("line 19\n", "MODIFIED 19\n");
    test_repo.write_file_content(file_name, &modified);

    // Stage only hunk 0 (the line 2 change); hunk 1 (line 19) remains unstaged
    stage_hunk(test_repo.repo_path(), "test.txt", 0).unwrap();

    // Build the model (unstaged section first, then staged section in the UI)
    let mut model = create_model_from_test_repo(&test_repo);

    // Remove staged file from collapsed_sections to expand it
    model
        .ui_model
        .collapsed_sections
        .remove(&SectionType::StagedFile {
            path: "test.txt".to_string(),
        });

    // Find the deletion line ("-line 2") in the staged hunk
    let deletion_pos = model
        .ui_model
        .lines
        .iter()
        .position(|l| {
            matches!(&l.content, LineContent::DiffLine(dl) if dl.content == "line 2")
                && matches!(&l.section, Some(SectionType::StagedHunk { path, .. }) if path == "test.txt")
        })
        .expect("staged deletion line should be present");

    // The addition line ("+MODIFIED 2") comes right after
    let addition_pos = deletion_pos + 1;
    assert!(
        matches!(
            &model.ui_model.lines[addition_pos].content,
            LineContent::DiffLine(dl) if dl.content == "MODIFIED 2"
        ),
        "addition line should follow the deletion line"
    );
    assert!(
        matches!(
            &model.ui_model.lines[addition_pos].section,
            Some(SectionType::StagedHunk { path, .. }) if path == "test.txt"
        ),
        "addition line should be in the staged hunk"
    );

    // Enter visual mode on the deletion line, extend to the addition line
    model.ui_model.cursor_position = deletion_pos;
    update(&mut model, Message::EnterVisualMode);
    model.ui_model.cursor_position = addition_pos;

    // Unstage selected lines
    let follow_up = update(&mut model, Message::UnstageSelected);
    assert_eq!(follow_up, Some(Message::Refresh));
    update(&mut model, Message::Refresh);

    // After unstaging, the staged section should be empty for test.txt
    let has_staged = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::StagedFile(fc) if fc.path == "test.txt"));
    assert!(
        !has_staged,
        "test.txt should no longer appear in staged changes after unstaging all its staged lines"
    );

    // The working tree should still have both modifications
    let wt_content = fs::read_to_string(test_repo.repo_path().join(&file_name)).unwrap();
    assert!(
        wt_content.contains("MODIFIED 2"),
        "working tree should still have MODIFIED 2"
    );
    assert!(
        wt_content.contains("MODIFIED 19"),
        "working tree should still have MODIFIED 19"
    );
}

#[test]
fn test_visual_unstage_single_staged_file() {
    let file_name = "test.txt";

    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_name)
        .stage_files(&[file_name])
        .commit("Add test.txt")
        .write_file_content(file_name, "modified test.txt")
        .stage_files(&[file_name]);

    let mut model = create_model_from_test_repo(&test_repo);

    let pos =
        find_staged_file_line(&model, file_name).expect("test.txt should be in staged changes");

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
        .any(|l| matches!(&l.content, LineContent::StagedFile(fc) if fc.path == file_name));
    let has_unstaged = model
        .ui_model
        .lines
        .iter()
        .any(|l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == file_name));

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
    let file_a = "file_a.txt";
    let file_b = "file_b.txt";
    let file_c = "file_c.txt";

    // Visual select file_a and file_b but NOT file_c.
    // Only file_a and file_b should be unstaged.
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_a)
        .create_file(file_b)
        .create_file(file_c)
        .stage_files(&[file_a, file_b, file_c])
        .commit("Commit 3 files");

    // Modify all three and stage them
    test_repo
        .write_file_content(file_a, "modified a")
        .write_file_content(file_b, "modified b")
        .write_file_content(file_c, "modified c")
        .stage_files(&[file_a, file_b, file_c]);

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
