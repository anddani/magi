use magi::git::discard::{
    discard_files, discard_hunk, discard_lines, discard_staged_files, discard_staged_hunk,
};
use magi::git::test_repo::TestRepo;
use std::fs;
use std::process::Command;

/// Helper function to commit staged changes in a test repository using CLI git
fn commit_changes(repo_path: &std::path::Path, message: &str) {
    let output = Command::new("git")
        .args(["-C", repo_path.to_str().unwrap(), "commit", "-m", message])
        .env("GIT_AUTHOR_NAME", "Test User")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "Test User")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .output()
        .expect("Failed to run git commit");
    assert!(
        output.status.success(),
        "git commit failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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
    commit_changes(repo_path, "Add test2.txt");

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
    commit_changes(repo_path, "Initial content");

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
    commit_changes(repo_path, "Add test2.txt");

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
    commit_changes(repo_path, "Initial content");

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

// ============================================================================
// Tests for staged discard operations
// ============================================================================

#[test]
fn test_discard_staged_modified_file() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Modify and stage a file
    let file_path = repo_path.join("test.txt");
    let original_content = fs::read_to_string(&file_path).unwrap();
    fs::write(&file_path, "staged modification").unwrap();
    magi::git::stage::stage_files(repo_path, &["test.txt"]).unwrap();

    // Verify file is staged
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_index_modified()),
        "File should be staged before discard"
    );

    // Discard staged changes - removes from BOTH index and working tree
    discard_staged_files(repo_path, &["test.txt"]).unwrap();

    // Verify file is no longer staged
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        !statuses.iter().any(|s| s.status().is_index_modified()),
        "File should not be staged after discard"
    );

    // Working tree should be reverted to original content
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(
        content, original_content,
        "Working tree should be reverted to original content"
    );
}

#[test]
fn test_discard_staged_preserves_unstaged_changes() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create a file with multiple lines and commit
    let file_path = repo_path.join("test.txt");
    let original_content = "line 1\nline 2\nline 3\nline 4\nline 5\n";
    fs::write(&file_path, original_content).unwrap();
    magi::git::stage::stage_files(repo_path, &["test.txt"]).unwrap();
    commit_changes(repo_path, "Initial content");

    // Stage a change to line 2
    let staged_content = "line 1\nSTAGED CHANGE\nline 3\nline 4\nline 5\n";
    fs::write(&file_path, staged_content).unwrap();
    magi::git::stage::stage_files(repo_path, &["test.txt"]).unwrap();

    // Then make an additional unstaged change to line 4
    let working_content = "line 1\nSTAGED CHANGE\nline 3\nUNSTAGED CHANGE\nline 5\n";
    fs::write(&file_path, working_content).unwrap();

    // Discard staged changes
    discard_staged_files(repo_path, &["test.txt"]).unwrap();

    // Staged change (line 2) should be reverted, unstaged change (line 4) preserved
    let content = fs::read_to_string(&file_path).unwrap();
    let expected = "line 1\nline 2\nline 3\nUNSTAGED CHANGE\nline 5\n";
    assert_eq!(
        content, expected,
        "Staged change should be discarded, unstaged preserved"
    );
}

#[test]
fn test_discard_staged_new_file_deletes_it() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create and stage a new file
    let file_path = repo_path.join("new_file.txt");
    fs::write(&file_path, "new file content").unwrap();
    magi::git::stage::stage_files(repo_path, &["new_file.txt"]).unwrap();

    // Verify file is staged as new
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_index_new()),
        "File should be staged as new before discard"
    );
    assert!(file_path.exists(), "File should exist before discard");

    // Discard staged new file - should delete it
    discard_staged_files(repo_path, &["new_file.txt"]).unwrap();

    // Verify file is no longer staged and deleted
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        !statuses.iter().any(|s| s.path() == Some("new_file.txt")),
        "File should not appear in status after discard"
    );
    assert!(!file_path.exists(), "File should be deleted after discard");
}

#[test]
fn test_discard_staged_hunk() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create a file with multiple lines
    let file_path = repo_path.join("test.txt");
    let original_content = "line 1\nline 2\nline 3\nline 4\nline 5\n";
    fs::write(&file_path, original_content).unwrap();
    magi::git::stage::stage_files(repo_path, &["test.txt"]).unwrap();
    commit_changes(repo_path, "Initial content");

    // Modify and stage
    let modified_content = "line 1\nSTAGED CHANGE\nline 3\nline 4\nline 5\n";
    fs::write(&file_path, modified_content).unwrap();
    magi::git::stage::stage_files(repo_path, &["test.txt"]).unwrap();

    // Verify file is staged
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_index_modified()),
        "File should be staged before discard_staged_hunk"
    );

    // Discard the staged hunk - removes from both index and working tree
    discard_staged_hunk(repo_path, "test.txt", 0).unwrap();

    // Verify file is no longer staged
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        !statuses.iter().any(|s| s.status().is_index_modified()),
        "File should not be staged after discard_staged_hunk"
    );

    // Working tree should be reverted to original content
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(
        content, original_content,
        "Working tree should be reverted to original content"
    );
}

#[test]
fn test_discard_staged_files_empty_list_is_noop() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Modify and stage a file
    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, "staged content").unwrap();
    magi::git::stage::stage_files(repo_path, &["test.txt"]).unwrap();

    // Discard with empty list
    discard_staged_files(repo_path, &[]).unwrap();

    // File should remain staged
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_index_modified()),
        "File should remain staged when discarding empty list"
    );
}
