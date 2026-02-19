use magi::git::stage::{stage_files, stage_hunk, unstage_files, unstage_lines};
use magi::git::test_repo::TestRepo;
use std::fs;
use std::process::Command;

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
fn test_stage_files_stages_modified_tracked_file() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Modify the tracked file
    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, "modified content").unwrap();

    // Verify file is modified but not staged
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(statuses.iter().any(|s| s.status().is_wt_modified()));

    // Stage the modified file
    stage_files(repo_path, &["test.txt"]).unwrap();

    // Refresh and verify file is now staged
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_index_modified()),
        "File should be staged after stage_files"
    );
}

#[test]
fn test_unstage_files_unstages_modified_file() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Modify and stage a file
    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, "modified content").unwrap();
    stage_files(repo_path, &["test.txt"]).unwrap();

    // Verify file is staged
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_index_modified()),
        "File should be staged before unstage_files"
    );

    // Unstage the file
    unstage_files(repo_path, &["test.txt"]).unwrap();

    // Verify file is no longer staged (but still modified in working tree)
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        !statuses.iter().any(|s| s.status().is_index_modified()),
        "File should not be staged after unstage_files"
    );
    assert!(
        statuses.iter().any(|s| s.status().is_wt_modified()),
        "File should still be modified in working tree"
    );
}

#[test]
fn test_stage_files_stages_specific_file() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create two untracked files
    fs::write(repo_path.join("file1.txt"), "content1").unwrap();
    fs::write(repo_path.join("file2.txt"), "content2").unwrap();

    // Stage only file1.txt
    stage_files(repo_path, &["file1.txt"]).unwrap();

    // Verify only file1 is staged
    let statuses = test_repo.repo.statuses(None).unwrap();
    for entry in statuses.iter() {
        let path = entry.path().unwrap();
        if path == "file1.txt" {
            assert!(entry.status().is_index_new(), "file1.txt should be staged");
        } else if path == "file2.txt" {
            assert!(
                entry.status().is_wt_new(),
                "file2.txt should remain untracked"
            );
        }
    }
}

#[test]
fn test_stage_files_stages_multiple_files() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create three untracked files
    fs::write(repo_path.join("file1.txt"), "content1").unwrap();
    fs::write(repo_path.join("file2.txt"), "content2").unwrap();
    fs::write(repo_path.join("file3.txt"), "content3").unwrap();

    // Stage file1 and file2
    stage_files(repo_path, &["file1.txt", "file2.txt"]).unwrap();

    // Verify file1 and file2 are staged, file3 is not
    let statuses = test_repo.repo.statuses(None).unwrap();
    for entry in statuses.iter() {
        let path = entry.path().unwrap();
        match path {
            "file1.txt" | "file2.txt" => {
                assert!(entry.status().is_index_new(), "{} should be staged", path);
            }
            "file3.txt" => {
                assert!(
                    entry.status().is_wt_new(),
                    "file3.txt should remain untracked"
                );
            }
            _ => {}
        }
    }
}

#[test]
fn test_stage_files_empty_list_is_noop() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create an untracked file
    fs::write(repo_path.join("file.txt"), "content").unwrap();

    // Stage with empty list
    stage_files(repo_path, &[]).unwrap();

    // File should remain untracked
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_wt_new()),
        "File should remain untracked when staging empty list"
    );
}

#[test]
fn test_unstage_files_unstages_specific_file() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create and stage two files
    fs::write(repo_path.join("file1.txt"), "content1").unwrap();
    fs::write(repo_path.join("file2.txt"), "content2").unwrap();
    stage_files(repo_path, &["file1.txt", "file2.txt"]).unwrap();

    // Unstage only file1
    unstage_files(repo_path, &["file1.txt"]).unwrap();

    // Verify file1 is unstaged, file2 is still staged
    let statuses = test_repo.repo.statuses(None).unwrap();
    for entry in statuses.iter() {
        let path = entry.path().unwrap();
        if path == "file1.txt" {
            assert!(
                entry.status().is_wt_new(),
                "file1.txt should be unstaged (untracked)"
            );
        } else if path == "file2.txt" {
            assert!(
                entry.status().is_index_new(),
                "file2.txt should remain staged"
            );
        }
    }
}

#[test]
fn test_unstage_files_empty_list_is_noop() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create and stage a file
    fs::write(repo_path.join("file.txt"), "content").unwrap();
    stage_files(repo_path, &["file.txt"]).unwrap();

    // Unstage with empty list
    unstage_files(repo_path, &[]).unwrap();

    // File should remain staged
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_index_new()),
        "File should remain staged when unstaging empty list"
    );
}

#[test]
fn test_unstage_lines_after_staging_hunk_with_two_hunks() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create a file with 20 lines (enough separation for two distinct hunks)
    let mut content = String::new();
    for i in 1..=20 {
        content.push_str(&format!("line {}\n", i));
    }
    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, &content).unwrap();
    stage_files(repo_path, &["test.txt"]).unwrap();
    commit_changes(repo_path, "Initial content with 20 lines");

    // Modify lines 2 and 19 to create two separate hunks
    let modified = content
        .replace("line 2\n", "MODIFIED 2\n")
        .replace("line 19\n", "MODIFIED 19\n");
    fs::write(&file_path, &modified).unwrap();

    // Stage hunk 0 (the change at line 2)
    stage_hunk(repo_path, "test.txt", 0).unwrap();

    // Now try to unstage specific lines from the staged hunk
    // The staged diff should have one hunk with:
    //   content_lines[0]: " line 1"      (context)
    //   content_lines[1]: "-line 2"       (deletion)
    //   content_lines[2]: "+MODIFIED 2"   (addition)
    //   content_lines[3]: " line 3"       (context)
    //   ...
    // Unstage the change (lines 1 and 2 = the - and + lines)
    unstage_lines(repo_path, "test.txt", 0, &[1, 2]).unwrap();

    // After unstaging, line 2 should be back to original in the index
    // The working tree still has MODIFIED 2
    let result = fs::read_to_string(&file_path).unwrap();
    assert!(
        result.contains("MODIFIED 2"),
        "Working tree should still have MODIFIED 2"
    );

    // The file should now show as both unstaged-modified (MODIFIED 2 in working tree)
    // and NOT staged anymore for that hunk
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        statuses.iter().any(|s| s.status().is_wt_modified()),
        "File should have unstaged modifications"
    );
}

#[test]
fn test_unstage_lines_no_trailing_newline() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create a file with 20 lines, NO trailing newline
    let mut lines_vec: Vec<String> = (1..=20).map(|i| format!("line {}", i)).collect();
    let content = lines_vec.join("\n"); // no trailing \n
    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, &content).unwrap();
    stage_files(repo_path, &["test.txt"]).unwrap();
    commit_changes(repo_path, "Initial content without trailing newline");

    // Modify lines 2 and 19 to create two separate hunks
    lines_vec[1] = "MODIFIED 2".to_string();
    lines_vec[18] = "MODIFIED 19".to_string();
    let modified = lines_vec.join("\n");
    fs::write(&file_path, &modified).unwrap();

    // Stage hunk 1 (the change at line 19, which is near the end - no trailing newline)
    stage_hunk(repo_path, "test.txt", 1).unwrap();

    // Unstage the change lines from the staged hunk.
    // The staged diff has "\ No newline at end of file" markers that must be
    // handled correctly. The UI indices skip these markers:
    //   ui_index 0: " line 16"       (context)
    //   ui_index 1: " line 17"       (context)
    //   ui_index 2: " line 18"       (context)
    //   ui_index 3: "-line 19"       (deletion)
    //   ui_index 4: "+MODIFIED 19"   (addition)
    //   ui_index 5: " line 20"       (context)
    unstage_lines(repo_path, "test.txt", 0, &[3, 4]).unwrap();

    // After unstaging, the index should no longer have the staged change
    let statuses = test_repo.repo.statuses(None).unwrap();
    assert!(
        !statuses.iter().any(|s| s.status().is_index_modified()),
        "File should not be staged after unstaging lines"
    );
}

#[test]
fn test_unstage_partial_lines_with_unstaged_hunk_present() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create a file with 20 lines
    let mut content = String::new();
    for i in 1..=20 {
        content.push_str(&format!("line {}\n", i));
    }
    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, &content).unwrap();
    stage_files(repo_path, &["test.txt"]).unwrap();
    commit_changes(repo_path, "Initial 20 lines");

    // Modify lines 2 and 19 (two separate hunks)
    let modified = content
        .replace("line 2\n", "MODIFIED 2\n")
        .replace("line 19\n", "MODIFIED 19\n");
    fs::write(&file_path, &modified).unwrap();

    // Stage hunk 0 (line 2 change), leaving hunk 1 (line 19) unstaged
    stage_hunk(repo_path, "test.txt", 0).unwrap();

    // Try to unstage ONLY the addition line (ui_index=2), not the deletion (ui_index=1)
    // The staged diff has:
    //   ui_index=0: " line 1"     (context)
    //   ui_index=1: "-line 2"     (deletion)
    //   ui_index=2: "+MODIFIED 2" (addition)
    //   ui_index=3: " line 3"     (context)
    //   ...
    unstage_lines(repo_path, "test.txt", 0, &[2]).unwrap();
}

#[test]
fn test_unstage_lines_second_staged_hunk_with_unstaged_hunk() {
    let test_repo = TestRepo::new();
    let repo_path = test_repo.repo.workdir().unwrap();

    // Create a file with 30 lines
    let mut content = String::new();
    for i in 1..=30 {
        content.push_str(&format!("line {}\n", i));
    }
    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, &content).unwrap();
    stage_files(repo_path, &["test.txt"]).unwrap();
    commit_changes(repo_path, "Initial 30 lines");

    // Modify lines 2, 15, and 28 (three separate hunks)
    let modified = content
        .replace("line 2\n", "MODIFIED 2\n")
        .replace("line 15\n", "MODIFIED 15\n")
        .replace("line 28\n", "MODIFIED 28\n");
    fs::write(&file_path, &modified).unwrap();

    // Stage hunks 0 and 1 (line 2 and line 15 changes), leaving hunk 2 (line 28) unstaged
    stage_hunk(repo_path, "test.txt", 0).unwrap();
    stage_hunk(repo_path, "test.txt", 1).unwrap();

    // Now unstage lines from the SECOND staged hunk (hunk_index=1, the line 15 change)
    // The staged diff now has two hunks; hunk 1 is the line 15 change
    // Staged diff hunk 1 content:
    //   ui_index=0: " line 12"    (context)
    //   ui_index=1: " line 13"    (context)
    //   ui_index=2: " line 14"    (context)
    //   ui_index=3: "-line 15"    (deletion)
    //   ui_index=4: "+MODIFIED 15"(addition)
    //   ui_index=5: " line 16"    (context)
    //   ui_index=6: " line 17"    (context)
    //   ui_index=7: " line 18"    (context)
    unstage_lines(repo_path, "test.txt", 1, &[3, 4]).unwrap();
}
