use magi::git::reverse::{
    reverse_patch, reverse_staged_files, reverse_staged_hunk, reverse_staged_hunks,
    reverse_staged_lines,
};
use magi::git::test_repo::TestRepo;
use std::fs;

fn file_content(test_repo: &TestRepo, file: &str) -> String {
    fs::read_to_string(test_repo.repo_path().join(file)).unwrap()
}

/// The staged change must survive a reverse: only the working tree is undone.
fn assert_still_staged(test_repo: &TestRepo) {
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_index_modified()),
        "Change should still be staged in the index after reverse"
    );
}

#[test]
fn test_reverse_patch_undoes_change_in_working_tree() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo.commit_file(file_name, "one\ntwo\n", "Initial content");

    let patch = "\
diff --git a/test.txt b/test.txt
--- a/test.txt
+++ b/test.txt
@@ -1,2 +1,2 @@
 one
-TWO
+two
";
    // The patch describes changing "TWO" to "two"; reversing it turns
    // the committed "two" back into "TWO".
    reverse_patch(test_repo.repo_path(), patch).unwrap();

    assert_eq!(file_content(&test_repo, file_name), "one\nTWO\n");
}

#[test]
fn test_reverse_staged_file_keeps_index_intact() {
    let file_name = "test.txt";
    let original_content = "line 1\nline 2\nline 3\n";
    let staged_content = "line 1\nMODIFIED\nline 3\n";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, original_content, "Initial content")
        .write_file_content(file_name, staged_content)
        .stage_files(&[file_name]);

    reverse_staged_files(test_repo.repo_path(), &[file_name]).unwrap();

    // Working tree is back to the committed content
    assert_eq!(file_content(&test_repo, file_name), original_content);
    // ...but the change is still staged
    assert_still_staged(&test_repo);
}

#[test]
fn test_reverse_staged_files_empty_list_is_noop() {
    let test_repo = TestRepo::new();
    test_repo.commit_file("test.txt", "content\n", "Initial content");

    reverse_staged_files(test_repo.repo_path(), &[]).unwrap();
}

#[test]
fn test_reverse_staged_files_skips_file_without_staged_changes() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, "content\n", "Initial content")
        .write_file_content(file_name, "unstaged change\n");

    // No staged changes: nothing to reverse, the unstaged change stays
    reverse_staged_files(test_repo.repo_path(), &[file_name]).unwrap();

    assert_eq!(file_content(&test_repo, file_name), "unstaged change\n");
}

/// Content with two far-apart modifications so the staged diff has two hunks.
fn setup_two_staged_hunks(test_repo: &TestRepo, file_name: &str) -> (String, String) {
    let original: String = (1..=20).map(|i| format!("line {}\n", i)).collect();
    let modified = original
        .replace("line 2\n", "MODIFIED 2\n")
        .replace("line 18\n", "MODIFIED 18\n");
    test_repo
        .commit_file(file_name, &original, "Initial content")
        .write_file_content(file_name, &modified)
        .stage_files(&[file_name]);
    (original, modified)
}

#[test]
fn test_reverse_staged_hunk_reverses_only_that_hunk() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    let (_, modified) = setup_two_staged_hunks(&test_repo, file_name);

    reverse_staged_hunk(test_repo.repo_path(), file_name, 0).unwrap();

    let content = file_content(&test_repo, file_name);
    // First hunk undone, second hunk still present
    assert!(content.contains("line 2\n"), "First hunk should be undone");
    assert!(
        content.contains("MODIFIED 18\n"),
        "Second hunk should remain"
    );
    assert_ne!(content, modified);
    assert_still_staged(&test_repo);
}

#[test]
fn test_reverse_staged_hunks_reverses_all_selected() {
    let file_name = "test.txt";
    let test_repo = TestRepo::new();
    let (original, _) = setup_two_staged_hunks(&test_repo, file_name);

    // Indices arrive in reverse order from visual selection
    reverse_staged_hunks(test_repo.repo_path(), file_name, &[1, 0]).unwrap();

    assert_eq!(file_content(&test_repo, file_name), original);
    assert_still_staged(&test_repo);
}

#[test]
fn test_reverse_staged_lines_reverses_only_selected_lines() {
    let file_name = "test.txt";
    let original_content = "line 1\nline 2\nline 3\n";
    let staged_content = "line 1\nADDED A\nADDED B\nline 2\nline 3\n";
    let test_repo = TestRepo::new();
    test_repo
        .commit_file(file_name, original_content, "Initial content")
        .write_file_content(file_name, staged_content)
        .stage_files(&[file_name]);

    // The hunk's diff lines are: " line 1", "+ADDED A", "+ADDED B", " line 2",
    // " line 3". Reverse only "+ADDED A" (index 1).
    reverse_staged_lines(test_repo.repo_path(), file_name, 0, &[1]).unwrap();

    assert_eq!(
        file_content(&test_repo, file_name),
        "line 1\nADDED B\nline 2\nline 3\n"
    );
    assert_still_staged(&test_repo);
}

#[test]
fn test_reverse_staged_new_file_removes_content_from_working_tree() {
    let file_name = "new.txt";
    let test_repo = TestRepo::new();
    // Need an initial commit so the staged diff is against HEAD
    test_repo
        .commit_file("other.txt", "content\n", "Initial commit")
        .write_file_content(file_name, "new content\n")
        .stage_files(&[file_name]);

    reverse_staged_files(test_repo.repo_path(), &[file_name]).unwrap();

    // Reversing the addition removes the file from the working tree,
    // while it stays in the index
    assert!(!test_repo.repo_path().join(file_name).exists());
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_index_new()),
        "New file should still be staged in the index"
    );
}
