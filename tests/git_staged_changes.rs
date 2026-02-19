use magi::git::staged_changes::get_lines;
use magi::git::test_repo::TestRepo;
use magi::model::{DiffLineType, FileStatus, LineContent};

#[test]
fn test_get_lines_with_staged_changes() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_name)
        .stage_files(&[file_name])
        .commit("Add test.txt")
        .write_file_content(file_name, "modified content\nwith new line")
        .stage_files(&[file_name]);

    let lines = get_lines(&test_repo.repo).unwrap();

    // Should have at least: header + file + hunk + diff lines
    assert!(lines.len() >= 3);

    // Check section header
    match &lines[0].content {
        LineContent::SectionHeader { title, count } => {
            assert_eq!(title, "Staged changes");
            assert_eq!(*count, Some(1));
        }
        _ => panic!("Expected SectionHeader"),
    }

    // Check staged file
    match &lines[1].content {
        LineContent::StagedFile(fc) => {
            assert_eq!(fc.path, "test.txt");
            assert_eq!(fc.status, FileStatus::Modified);
        }
        _ => panic!("Expected StagedFile"),
    }

    // Check that there's a hunk header
    match &lines[2].content {
        LineContent::DiffHunk(hunk) => {
            assert!(hunk.header.starts_with("@@"));
        }
        _ => panic!("Expected DiffHunk"),
    }
}

#[test]
fn test_get_lines_no_staged_changes() {
    let test_repo = TestRepo::new();

    // No staging, so no staged changes
    let lines = get_lines(&test_repo.repo).unwrap();

    assert_eq!(lines.len(), 0);
}

#[test]
fn test_get_lines_with_new_staged_file() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo.create_file(file_name).stage_files(&[file_name]);

    let lines = get_lines(&test_repo.repo).unwrap();

    // Should have header + file + hunk + diff lines
    assert!(lines.len() >= 3);

    // Check staged file status
    match &lines[1].content {
        LineContent::StagedFile(fc) => {
            assert_eq!(fc.path, file_name);
            assert_eq!(fc.status, FileStatus::New);
        }
        _ => panic!("Expected StagedFile"),
    }
}

#[test]
fn test_diff_lines_have_correct_types() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_name)
        .stage_files(&[file_name])
        .commit("Add test.txt")
        .write_file_content(file_name, "new content")
        .stage_files(&[file_name]);

    let lines = get_lines(&test_repo.repo).unwrap();

    // Find diff lines and verify they have correct types
    let diff_lines: Vec<_> = lines
        .iter()
        .filter_map(|line| {
            if let LineContent::DiffLine(dl) = &line.content {
                Some(dl)
            } else {
                None
            }
        })
        .collect();

    // Should have both additions and deletions
    assert!(
        diff_lines
            .iter()
            .any(|dl| dl.line_type == DiffLineType::Deletion)
    );
    assert!(
        diff_lines
            .iter()
            .any(|dl| dl.line_type == DiffLineType::Addition)
    );
}
