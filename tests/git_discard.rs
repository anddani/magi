use git2::Signature;
use magi::git::discard::{discard_files, discard_hunk, discard_lines};
use magi::git::test_repo::TestRepo;
use std::fs;

/// Helper function to commit staged changes in a test repository
fn commit_changes(repo: &git2::Repository, message: &str) {
    let signature = Signature::now("Test User", "test@example.com").unwrap();
    let mut index = repo.index().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();

    let parent_commit = repo.head().ok().and_then(|head| head.peel_to_commit().ok());

    let parents: Vec<&git2::Commit> = parent_commit.iter().collect();

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parents,
    )
    .unwrap();
}

#[test]
fn test_discard_single_file() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Modify the tracked file
    let file_path = repo_path.join("test.txt");
    let original_content = fs::read_to_string(&file_path).unwrap();
    fs::write(&file_path, "modified content").unwrap();

    // Verify file is modified
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_wt_modified()),
        "File should be modified before discard"
    );

    // Discard changes
    discard_files(repo_path, &["test.txt"]).unwrap();

    // Verify file is no longer modified
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        !statuses.iter().any(|s| s.status().is_wt_modified()),
        "File should not be modified after discard"
    );

    // Verify content is restored
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, original_content, "Content should be restored");
}

#[test]
fn test_discard_multiple_files() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create and commit another file
    let file2_path = repo_path.join("test2.txt");
    fs::write(&file2_path, "initial content 2").unwrap();
    magi::git::stage::stage_files(repo_path, &["test2.txt"]).unwrap();
    commit_changes(&test_repo.repo, "Add test2.txt");

    // Modify both files
    let file1_path = repo_path.join("test.txt");
    fs::write(&file1_path, "modified 1").unwrap();
    fs::write(&file2_path, "modified 2").unwrap();

    // Verify both files are modified
    let statuses = test_repo.repo.statuses(None).unwrap();
    let modified_count = statuses
        .iter()
        .filter(|s| s.status().is_wt_modified())
        .count();
    assert_eq!(modified_count, 2, "Both files should be modified");

    // Discard both files
    discard_files(repo_path, &["test.txt", "test2.txt"]).unwrap();

    // Verify no files are modified
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        !statuses.iter().any(|s| s.status().is_wt_modified()),
        "No files should be modified after discard"
    );
}

#[test]
fn test_discard_hunk() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create a file with multiple lines
    let file_path = repo_path.join("test.txt");
    let original_content = "line 1\nline 2\nline 3\nline 4\nline 5\n";
    fs::write(&file_path, original_content).unwrap();
    magi::git::stage::stage_files(repo_path, &["test.txt"]).unwrap();
    commit_changes(&test_repo.repo, "Initial content");

    // Modify the file to create a hunk
    let modified_content = "line 1\nMODIFIED\nline 3\nline 4\nline 5\n";
    fs::write(&file_path, modified_content).unwrap();

    // Verify file is modified
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_wt_modified()),
        "File should be modified before discard_hunk"
    );

    // Discard the hunk
    discard_hunk(repo_path, "test.txt", 0).unwrap();

    // Verify file is no longer modified
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        !statuses.iter().any(|s| s.status().is_wt_modified()),
        "File should not be modified after discard_hunk"
    );

    // Verify content is restored
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, original_content, "Content should be restored");
}

#[test]
fn test_discard_files_empty_list_is_noop() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Modify a file
    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, "modified content").unwrap();

    // Discard with empty list
    discard_files(repo_path, &[]).unwrap();

    // File should remain modified
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_wt_modified()),
        "File should remain modified when discarding empty list"
    );
}

#[test]
fn test_discard_specific_file_leaves_other_unchanged() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create and commit another file
    let file2_path = repo_path.join("test2.txt");
    fs::write(&file2_path, "initial content 2").unwrap();
    magi::git::stage::stage_files(repo_path, &["test2.txt"]).unwrap();
    commit_changes(&test_repo.repo, "Add test2.txt");

    // Modify both files
    let file1_path = repo_path.join("test.txt");
    fs::write(&file1_path, "modified 1").unwrap();
    fs::write(&file2_path, "modified 2").unwrap();

    // Discard only test.txt
    discard_files(repo_path, &["test.txt"]).unwrap();

    // Verify test.txt is restored and test2.txt is still modified
    let statuses = test_repo.repo.statuses(None).unwrap();
    for entry in statuses.iter() {
        let path = entry.path().unwrap();
        if path == "test.txt" {
            assert!(
                !entry.status().is_wt_modified(),
                "test.txt should not be modified after discard"
            );
        } else if path == "test2.txt" {
            assert!(
                entry.status().is_wt_modified(),
                "test2.txt should still be modified"
            );
        }
    }
}

#[test]
fn test_discard_lines() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create a file with multiple lines
    let file_path = repo_path.join("test.txt");
    let original_content = "line 1\nline 2\nline 3\nline 4\nline 5\n";
    fs::write(&file_path, original_content).unwrap();
    magi::git::stage::stage_files(repo_path, &["test.txt"]).unwrap();
    commit_changes(&test_repo.repo, "Initial content");

    // Modify multiple lines
    let modified_content = "line 1\nMODIFIED 2\nMODIFIED 3\nline 4\nline 5\n";
    fs::write(&file_path, modified_content).unwrap();

    // Discard only the first modified line (line index 0 in the hunk's diff lines)
    // The diff will have: - line 2\n + MODIFIED 2\n - line 3\n + MODIFIED 3
    // We want to discard line 2's change (indices 0 and 1 in the content lines)
    discard_lines(repo_path, "test.txt", 0, &[0, 1]).unwrap();

    // Read the result
    let content = fs::read_to_string(&file_path).unwrap();
    // After discarding lines 0 and 1, we expect line 2 to be restored
    // while line 3 remains modified
    assert!(
        content.contains("line 2"),
        "line 2 should be restored after discard_lines"
    );
    assert!(
        content.contains("MODIFIED 3"),
        "MODIFIED 3 should remain since we only discarded indices 0 and 1"
    );
}
