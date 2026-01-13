use magi::git::test_repo::TestRepo;
use magi::git::untracked_files::get_lines;
use magi::model::LineContent;
use std::fs;

#[test]
fn test_get_lines_with_untracked_files() {
    let test_repo = TestRepo::new();

    // Create an untracked file
    fs::write(
        test_repo.repo.workdir().unwrap().join("new_untracked.txt"),
        "test content",
    )
    .unwrap();

    let lines = get_lines(&test_repo.repo).unwrap();

    // Should have 2 lines: header + 1 file
    assert_eq!(lines.len(), 2);

    // Check section header
    match &lines[0].content {
        LineContent::SectionHeader { title, count } => {
            assert_eq!(title, "Untracked files");
            assert_eq!(*count, Some(1));
        }
        _ => panic!("Expected SectionHeader"),
    }

    // Check untracked file
    match &lines[1].content {
        LineContent::UntrackedFile(path) => {
            assert_eq!(path, "new_untracked.txt");
        }
        _ => panic!("Expected UntrackedFile"),
    }
}

#[test]
fn test_get_lines_no_untracked_files() {
    let test_repo = TestRepo::new();

    // The test repo commits test.txt, so there should be no untracked files
    let lines = get_lines(&test_repo.repo).unwrap();

    // Should have no lines when no untracked files (test.txt is committed)
    assert_eq!(lines.len(), 0);
}
