use magi::git::test_repo::TestRepo;
use magi::model::LineContent;
use magi::msg::Message;
use magi::msg::update::update;

mod utils;

use crate::utils::create_model_from_test_repo;

/// Find the line index for an UnstagedFile with the given path.
fn find_unstaged_file_line(model: &magi::model::Model, path: &str) -> Option<usize> {
    model
        .ui_model
        .lines
        .iter()
        .position(|l| matches!(&l.content, LineContent::UnstagedFile(fc) if fc.path == path))
}

/// Find the line index for a StagedFile with the given path.
fn find_staged_file_line(model: &magi::model::Model, path: &str) -> Option<usize> {
    model
        .ui_model
        .lines
        .iter()
        .position(|l| matches!(&l.content, LineContent::StagedFile(fc) if fc.path == path))
}

#[test]
fn test_cursor_moves_to_next_file_when_staging_first_file() {
    // Create a repository with two unstaged files
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

    // Position cursor on file_a
    let pos_a =
        find_unstaged_file_line(&model, file_a).expect("file_a.txt should be in unstaged changes");
    model.ui_model.cursor_position = pos_a;

    // Stage file_a
    update(&mut model, Message::StageSelected);
    update(&mut model, Message::Refresh);

    // After staging file_a, the cursor should be on file_b (the next unstaged file)
    let current_line = &model.ui_model.lines[model.ui_model.cursor_position];
    assert!(
        matches!(&current_line.content, LineContent::UnstagedFile(fc) if fc.path == file_b),
        "Cursor should be on file_b after staging file_a, but is on: {:?}",
        current_line.content
    );
}

#[test]
fn test_cursor_moves_to_prev_file_when_staging_last_file() {
    // Create a repository with two unstaged files
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

    // Position cursor on file_b (the last unstaged file)
    let pos_b =
        find_unstaged_file_line(&model, file_b).expect("file_b.txt should be in unstaged changes");
    model.ui_model.cursor_position = pos_b;

    // Stage file_b
    update(&mut model, Message::StageSelected);
    update(&mut model, Message::Refresh);

    // After staging file_b, the cursor should be on file_a (the previous/only remaining unstaged file)
    let current_line = &model.ui_model.lines[model.ui_model.cursor_position];
    assert!(
        matches!(&current_line.content, LineContent::UnstagedFile(fc) if fc.path == file_a),
        "Cursor should be on file_a after staging file_b, but is on: {:?}",
        current_line.content
    );
}

#[test]
fn test_cursor_stays_on_file_when_unstaging_it() {
    // Create a repository with staged files
    let file_a = "file_a.txt";
    let file_b = "file_b.txt";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_a)
        .create_file(file_b)
        .stage_files(&[file_a, file_b])
        .commit("Add files")
        .write_file_content(file_a, "modified a")
        .write_file_content(file_b, "modified b")
        .stage_files(&[file_a, file_b]);

    let mut model = create_model_from_test_repo(&test_repo);

    // Position cursor on staged file_a
    let pos_a =
        find_staged_file_line(&model, file_a).expect("file_a.txt should be in staged changes");
    model.ui_model.cursor_position = pos_a;

    // Unstage file_a
    update(&mut model, Message::UnstageSelected);
    update(&mut model, Message::Refresh);

    // After unstaging file_a, the cursor should move to the next staged file (file_b)
    // since file_a moved to unstaged section
    let current_line = &model.ui_model.lines[model.ui_model.cursor_position];
    assert!(
        matches!(&current_line.content, LineContent::StagedFile(fc) if fc.path == file_b),
        "Cursor should be on file_b (next staged file) after unstaging file_a, but is on: {:?}",
        current_line.content
    );
}

#[test]
fn test_cursor_moves_to_section_header_when_all_files_staged() {
    // Create a repository with one unstaged file
    let file_a = "file_a.txt";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_a)
        .stage_files(&[file_a])
        .commit("Add file")
        .write_file_content(file_a, "modified a");

    let mut model = create_model_from_test_repo(&test_repo);

    // Position cursor on the only unstaged file
    let pos_a =
        find_unstaged_file_line(&model, file_a).expect("file_a.txt should be in unstaged changes");
    model.ui_model.cursor_position = pos_a;

    // Stage file_a (the only unstaged file)
    update(&mut model, Message::StageSelected);
    update(&mut model, Message::Refresh);

    // After staging the last unstaged file, the cursor should be on a section header
    // (either the unstaged section header or the next section)
    let current_line = &model.ui_model.lines[model.ui_model.cursor_position];
    assert!(
        matches!(
            &current_line.content,
            LineContent::SectionHeader { .. } | LineContent::StagedFile(_)
        ),
        "Cursor should be on a section header or staged file after staging all unstaged files, but is on: {:?}",
        current_line.content
    );
}

#[test]
fn test_cursor_moves_to_untracked_section_when_all_unstaged_files_staged() {
    // Create a repository with untracked files and unstaged files
    let untracked = "untracked.txt";
    let unstaged = "unstaged.txt";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(unstaged)
        .stage_files(&[unstaged])
        .commit("Add unstaged file")
        .write_file_content(unstaged, "modified")
        .create_file(untracked); // Create untracked file after commit

    let mut model = create_model_from_test_repo(&test_repo);

    // Position cursor on the unstaged file
    let pos = find_unstaged_file_line(&model, unstaged)
        .expect("unstaged.txt should be in unstaged changes");
    model.ui_model.cursor_position = pos;

    // Stage the unstaged file
    update(&mut model, Message::StageSelected);
    update(&mut model, Message::Refresh);

    // After staging all unstaged files, cursor should move to the untracked section (above)
    let current_line = &model.ui_model.lines[model.ui_model.cursor_position];
    assert!(
        matches!(
            &current_line.content,
            LineContent::UntrackedFile(path) if path == untracked
        ) || matches!(&current_line.content, LineContent::SectionHeader { title, .. } if title.contains("Untracked")),
        "Cursor should be in untracked section after staging all unstaged files, but is on: {:?}",
        current_line.content
    );
}

#[test]
fn test_cursor_moves_to_unstaged_section_when_all_staged_files_unstaged() {
    // Create a repository with staged and unstaged files
    let file_a = "file_a.txt";
    let file_b = "file_b.txt";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_a)
        .create_file(file_b)
        .stage_files(&[file_a, file_b])
        .commit("Add files")
        .write_file_content(file_a, "modified a")
        .write_file_content(file_b, "modified b")
        // Stage only file_b, leaving file_a unstaged
        .stage_files(&[file_b]);

    let mut model = create_model_from_test_repo(&test_repo);

    // Position cursor on the staged file
    let pos =
        find_staged_file_line(&model, file_b).expect("file_b.txt should be in staged changes");
    model.ui_model.cursor_position = pos;

    // Unstage file_b (the only staged file)
    update(&mut model, Message::UnstageSelected);
    update(&mut model, Message::Refresh);

    // After unstaging all staged files, cursor should move to the unstaged section (above)
    let current_line = &model.ui_model.lines[model.ui_model.cursor_position];
    assert!(
        matches!(
            &current_line.content,
            LineContent::UnstagedFile(fc) if fc.path == file_a || fc.path == file_b
        ) || matches!(&current_line.content, LineContent::SectionHeader { title, .. } if title.contains("Unstaged")),
        "Cursor should be in unstaged section after unstaging all staged files, but is on: {:?}",
        current_line.content
    );
}

#[test]
fn test_cursor_moves_to_unstaged_when_all_untracked_files_staged() {
    // Create a repository with untracked and unstaged files
    let untracked = "untracked.txt";
    let unstaged = "unstaged.txt";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(unstaged)
        .stage_files(&[unstaged])
        .commit("Add file")
        .write_file_content(unstaged, "modified")
        .create_file(untracked);

    let mut model = create_model_from_test_repo(&test_repo);

    // Position cursor on the untracked file
    let pos = model
        .ui_model
        .lines
        .iter()
        .position(|l| matches!(&l.content, LineContent::UntrackedFile(p) if p == untracked))
        .expect("untracked.txt should exist");
    model.ui_model.cursor_position = pos;

    // Stage the untracked file
    update(&mut model, Message::StageSelected);
    update(&mut model, Message::Refresh);

    // After staging all untracked files, cursor should stay in place or move to unstaged section
    let current_line = &model.ui_model.lines[model.ui_model.cursor_position];
    assert!(
        matches!(
            &current_line.content,
            LineContent::UnstagedFile(fc) if fc.path == unstaged
        ) || matches!(&current_line.content, LineContent::SectionHeader { .. }),
        "Cursor should be in unstaged or section header after staging all untracked files, but is on: {:?}",
        current_line.content
    );
}
