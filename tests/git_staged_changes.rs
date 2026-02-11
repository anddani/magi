use magi::git::staged_changes::get_lines;
use magi::git::test_repo::TestRepo;
use magi::model::{DiffLineType, FileStatus, LineContent};
use std::fs;

#[test]
fn test_get_lines_with_staged_changes() {
    let test_repo = TestRepo::new();

    // Modify the existing tracked file
    let file_path = test_repo.repo.workdir().unwrap().join("test.txt");
    fs::write(&file_path, "modified content\nwith new line").unwrap();

    // Stage the changes
    let mut index = test_repo.repo.index().unwrap();
    index.add_path(std::path::Path::new("test.txt")).unwrap();
    index.write().unwrap();

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
    let test_repo = TestRepo::new();

    // Create a new file
    let file_path = test_repo.repo.workdir().unwrap().join("new_file.txt");
    fs::write(&file_path, "new file content").unwrap();

    // Stage the new file
    let mut index = test_repo.repo.index().unwrap();
    index
        .add_path(std::path::Path::new("new_file.txt"))
        .unwrap();
    index.write().unwrap();

    let lines = get_lines(&test_repo.repo).unwrap();

    // Should have header + file + hunk + diff lines
    assert!(lines.len() >= 3);

    // Check staged file status
    match &lines[1].content {
        LineContent::StagedFile(fc) => {
            assert_eq!(fc.path, "new_file.txt");
            assert_eq!(fc.status, FileStatus::New);
        }
        _ => panic!("Expected StagedFile"),
    }
}

#[test]
fn test_diff_lines_have_correct_types() {
    let test_repo = TestRepo::new();

    // Modify the file
    let file_path = test_repo.repo.workdir().unwrap().join("test.txt");
    fs::write(&file_path, "new content").unwrap();

    // Stage the changes
    let mut index = test_repo.repo.index().unwrap();
    index.add_path(std::path::Path::new("test.txt")).unwrap();
    index.write().unwrap();

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
