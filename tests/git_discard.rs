use magi::git::discard::{
    discard_files, discard_hunk, discard_lines, discard_staged_files, discard_staged_hunk,
    discard_untracked_files,
};
use magi::git::test_repo::TestRepo;
use std::fs;

#[test]
fn test_discard_single_file() {
    let file_name = "test.txt";
    let original_content = "original content";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_name)
        .write_file_content(file_name, original_content)
        .stage_files(&[file_name])
        .write_file_content(file_name, "modified content");

    // Verify file is modified
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_wt_modified()),
        "File should be modified before discard"
    );

    // Discard changes
    discard_files(test_repo.repo_path(), &["test.txt"]).unwrap();

    // Verify file is no longer modified
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        !statuses.iter().any(|s| s.status().is_wt_modified()),
        "File should not be modified after discard"
    );

    // Verify content is restored
    let content = fs::read_to_string(test_repo.repo_path().join(file_name)).unwrap();
    assert_eq!(content, original_content, "Content should be restored");
}

#[test]
fn test_discard_multiple_files() {
    let file_a = "file_a.txt";
    let file_b = "file_b.txt";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_a)
        .create_file(file_b)
        .stage_files(&[file_a, file_b])
        .commit("Add file_a.txt and file_b.txt")
        .write_file_content(file_a, "modified a")
        .write_file_content(file_b, "modified b");

    // Verify both files are modified
    let statuses = test_repo.repo.statuses(None).unwrap();
    let modified_count = statuses
        .iter()
        .filter(|s| s.status().is_wt_modified())
        .count();
    assert_eq!(modified_count, 2, "Both files should be modified");

    // Discard both files
    discard_files(test_repo.repo_path(), &[file_a, file_b]).unwrap();

    // Verify no files are modified
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        !statuses.iter().any(|s| s.status().is_wt_modified()),
        "No files should be modified after discard"
    );
}

#[test]
fn test_discard_hunk() {
    let file_name = "test.txt";
    let original_content = "line 1\nline 2\nline 3\nline 4\nline 5\n";
    let modified_content = "line 1\nMODIFIED\nline 3\nline 4\nline 5\n";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_name)
        .write_file_content(file_name, original_content)
        .stage_files(&[file_name])
        .commit("Initial content")
        .write_file_content(file_name, modified_content);

    // Verify file is modified
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_wt_modified()),
        "File should be modified before discard_hunk"
    );

    // Discard the hunk
    discard_hunk(test_repo.repo_path(), "test.txt", 0).unwrap();

    // Verify file is no longer modified
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        !statuses.iter().any(|s| s.status().is_wt_modified()),
        "File should not be modified after discard_hunk"
    );

    // Verify content is restored
    let content = fs::read_to_string(test_repo.repo_path().join(file_name)).unwrap();
    assert_eq!(content, original_content, "Content should be restored");
}

#[test]
fn test_discard_files_empty_list_is_noop() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_name)
        .stage_files(&[file_name])
        .commit("Add test.txt")
        .write_file_content(file_name, "modified content");

    // Discard with empty list
    discard_files(test_repo.repo_path(), &[]).unwrap();

    // File should remain modified
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_wt_modified()),
        "File should remain modified when discarding empty list"
    );
}

#[test]
fn test_discard_specific_file_leaves_other_unchanged() {
    let file_a = "file_a.txt";
    let file_b = "file_b.txt";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_a)
        .create_file(file_b)
        .write_file_content(file_a, "initial content a")
        .write_file_content(file_b, "initial content b")
        .stage_files(&[file_a, file_b])
        .commit("Add file a and b")
        .write_file_content(file_a, "modified a")
        .write_file_content(file_b, "modified b");

    // Discard only test.txt
    discard_files(test_repo.repo_path(), &[file_a]).unwrap();

    // Verify test.txt is restored and test2.txt is still modified
    let statuses = test_repo.repo.statuses(None).unwrap();
    for entry in statuses.iter() {
        let path = entry.path().unwrap();
        if path == file_a {
            assert!(
                !entry.status().is_wt_modified(),
                "file_a.txt should not be modified after discard"
            );
        } else if path == file_b {
            assert!(
                entry.status().is_wt_modified(),
                "file_b.txt should still be modified"
            );
        }
    }
}

#[test]
fn test_discard_lines() {
    let file_name = "test.txt";
    let original_content = "line 1\nline 2\nline 3\nline 4\nline 5\n";
    let modified_content = "line 1\nMODIFIED 2\nMODIFIED 3\nline 4\nline 5\n";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_name)
        .write_file_content(file_name, original_content)
        .stage_files(&[file_name])
        .commit("Initial content")
        .write_file_content(file_name, modified_content);

    // Discard only the first modified line (line index 0 in the hunk's diff lines)
    // The diff will have: - line 2\n + MODIFIED 2\n - line 3\n + MODIFIED 3
    // We want to discard line 2's change (indices 0 and 1 in the content lines)
    discard_lines(test_repo.repo_path(), "test.txt", 0, &[0, 1]).unwrap();

    // Read the result
    let content = fs::read_to_string(test_repo.repo_path().join(file_name)).unwrap();
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
    let file_name = "test.txt";
    let original_content = "original content";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_name)
        .write_file_content(file_name, original_content)
        .stage_files(&[file_name])
        .commit("Add test.txt")
        .write_file_content(file_name, "staged modification")
        .stage_files(&[file_name]);

    // Verify file is staged
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_index_modified()),
        "File should be staged before discard"
    );

    // Discard staged changes - removes from BOTH index and working tree
    discard_staged_files(test_repo.repo_path(), &[file_name]).unwrap();

    // Verify file is no longer staged
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        !statuses.iter().any(|s| s.status().is_index_modified()),
        "File should not be staged after discard"
    );

    // Working tree should be reverted to original content
    let content = fs::read_to_string(test_repo.repo_path().join(file_name)).unwrap();
    assert_eq!(
        content, original_content,
        "Working tree should be reverted to original content"
    );
}

#[test]
fn test_discard_staged_preserves_unstaged_changes() {
    let file_name = "test.txt";
    let original_content = "line 1\nline 2\nline 3\nline 4\nline 5\n";
    let staged_content = "line 1\nSTAGED CHANGE\nline 3\nline 4\nline 5\n";
    let working_content = "line 1\nSTAGED CHANGE\nline 3\nUNSTAGED CHANGE\nline 5\n";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_name)
        .write_file_content(file_name, original_content)
        .stage_files(&[file_name])
        .commit("Initial content")
        .write_file_content(file_name, staged_content)
        .stage_files(&[file_name])
        .write_file_content(file_name, working_content);

    // Discard staged changes
    discard_staged_files(test_repo.repo_path(), &["test.txt"]).unwrap();

    // Staged change (line 2) should be reverted, unstaged change (line 4) preserved
    let content = fs::read_to_string(test_repo.repo_path().join(file_name)).unwrap();
    let expected = "line 1\nline 2\nline 3\nUNSTAGED CHANGE\nline 5\n";
    assert_eq!(
        content, expected,
        "Staged change should be discarded, unstaged preserved"
    );
}

#[test]
fn test_discard_staged_new_file_deletes_it() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_name)
        .write_file_content(file_name, "original content")
        .stage_files(&[file_name]);

    // Verify file is staged as new
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_index_new()),
        "File should be staged as new before discard"
    );
    assert!(
        test_repo.repo_path().join(file_name).exists(),
        "File should exist before discard"
    );

    // Discard staged new file - should delete it
    discard_staged_files(test_repo.repo_path(), &[file_name]).unwrap();

    // Verify file is no longer staged and deleted
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        !statuses.iter().any(|s| s.path() == Some("new_file.txt")),
        "File should not appear in status after discard"
    );
    assert!(
        !test_repo.repo_path().join(file_name).exists(),
        "File should be deleted after discard"
    );
}

#[test]
fn test_discard_staged_hunk() {
    let file_name = "test.txt";
    let original_content = "line 1\nline 2\nline 3\nline 4\nline 5\n";
    let modified_content = "line 1\nSTAGED CHANGE\nline 3\nline 4\nline 5\n";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_name)
        .write_file_content(file_name, original_content)
        .stage_files(&[file_name])
        .commit("Initial content")
        .write_file_content(file_name, modified_content)
        .stage_files(&[file_name]);

    // Verify file is staged
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_index_modified()),
        "File should be staged before discard_staged_hunk"
    );

    // Discard the staged hunk - removes from both index and working tree
    discard_staged_hunk(test_repo.repo_path(), "test.txt", 0).unwrap();

    // Verify file is no longer staged
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        !statuses.iter().any(|s| s.status().is_index_modified()),
        "File should not be staged after discard_staged_hunk"
    );

    // Working tree should be reverted to original content
    let content = fs::read_to_string(test_repo.repo_path().join(file_name)).unwrap();
    assert_eq!(
        content, original_content,
        "Working tree should be reverted to original content"
    );
}

#[test]
fn test_discard_staged_files_empty_list_is_noop() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .create_file(file_name)
        .stage_files(&[file_name])
        .commit("Add test.txt")
        .write_file_content(file_name, "staged content")
        .stage_files(&[file_name]);

    // Discard with empty list
    discard_staged_files(test_repo.repo_path(), &[]).unwrap();

    // File should remain staged
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_index_modified()),
        "File should remain staged when discarding empty list"
    );
}

// ============================================================================
// Tests for untracked file discard operations
// ============================================================================

#[test]
fn test_discard_untracked_file() {
    let file_name = "untracked.txt";
    let test_repo = TestRepo::new();
    test_repo.create_file(file_name);

    // Create an untracked file

    // Verify file exists and is untracked
    assert!(
        test_repo.repo_path().join(file_name).exists(),
        "File should exist before discard"
    );
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_wt_new()),
        "File should be untracked before discard"
    );

    // Discard untracked file
    discard_untracked_files(test_repo.repo_path(), &[file_name]).unwrap();

    // Verify file is deleted
    assert!(
        !test_repo.repo_path().join(file_name).exists(),
        "File should be deleted after discard"
    );
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        !statuses.iter().any(|s| s.path() == Some(file_name)),
        "File should not appear in status after discard"
    );
}

#[test]
fn test_discard_untracked_files_empty_list_is_noop() {
    let file_name = "untracked.txt";
    let test_repo = TestRepo::new();
    test_repo.create_file(file_name);

    // Discard with empty list
    discard_untracked_files(test_repo.repo_path(), &[]).unwrap();

    // File should still exist
    assert!(
        test_repo.repo_path().join(file_name).exists(),
        "File should remain when discarding empty list"
    );
}
