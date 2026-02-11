use magi::git::test_repo::TestRepo;
use magi::git::unstaged_changes::get_lines;
use magi::model::{DiffLineType, FileStatus, LineContent};
use std::fs;

#[test]
fn test_get_lines_with_unstaged_changes() {
    let test_repo = TestRepo::new();

    // Modify the existing tracked file
    let file_path = test_repo.repo.workdir().unwrap().join("test.txt");
    fs::write(&file_path, "modified content\nwith new line").unwrap();

    let lines = get_lines(&test_repo.repo).unwrap();

    // Should have at least: header + file + hunk + diff lines
    assert!(lines.len() >= 3);

    // Check section header
    match &lines[0].content {
        LineContent::SectionHeader { title, count } => {
            assert_eq!(title, "Unstaged changes");
            assert_eq!(*count, Some(1));
        }
        _ => panic!("Expected SectionHeader"),
    }

    // Check unstaged file
    match &lines[1].content {
        LineContent::UnstagedFile(fc) => {
            assert_eq!(fc.path, "test.txt");
            assert_eq!(fc.status, FileStatus::Modified);
        }
        _ => panic!("Expected UnstagedFile"),
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
fn test_get_lines_no_unstaged_changes() {
    let test_repo = TestRepo::new();

    // No modifications, so no unstaged changes
    let lines = get_lines(&test_repo.repo).unwrap();

    assert_eq!(lines.len(), 0);
}

#[test]
fn test_get_lines_with_deleted_file() {
    let test_repo = TestRepo::new();

    // Delete the tracked file
    let file_path = test_repo.repo.workdir().unwrap().join("test.txt");
    fs::remove_file(&file_path).unwrap();

    let lines = get_lines(&test_repo.repo).unwrap();

    // Should have header + file + hunk + deletion lines
    assert!(lines.len() >= 3);

    // Check unstaged file status
    match &lines[1].content {
        LineContent::UnstagedFile(fc) => {
            assert_eq!(fc.path, "test.txt");
            assert_eq!(fc.status, FileStatus::Deleted);
        }
        _ => panic!("Expected UnstagedFile"),
    }
}

#[test]
fn test_diff_lines_have_correct_types() {
    let test_repo = TestRepo::new();

    // Modify the file
    let file_path = test_repo.repo.workdir().unwrap().join("test.txt");
    fs::write(&file_path, "new content").unwrap();

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
