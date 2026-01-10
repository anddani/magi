use std::path::Path;
use std::process::{Command, Stdio};

use crate::errors::MagiResult;

/// Stages the specified files.
/// If `files` is empty, this is a no-op.
pub fn stage_files<P: AsRef<Path>>(repo_path: P, files: &[&str]) -> MagiResult<()> {
    if files.is_empty() {
        return Ok(());
    }
    let _output = Command::new("git")
        .arg("-C")
        .arg(repo_path.as_ref())
        .arg("add")
        .arg("--")
        .args(files)
        .stdout(Stdio::piped())
        .output()?;
    Ok(())
}

/// Unstages the specified files.
/// If `files` is empty, this is a no-op.
pub fn unstage_files<P: AsRef<Path>>(repo_path: P, files: &[&str]) -> MagiResult<()> {
    if files.is_empty() {
        return Ok(());
    }
    let _output = Command::new("git")
        .arg("-C")
        .arg(repo_path.as_ref())
        .arg("reset")
        .arg("HEAD")
        .arg("--")
        .args(files)
        .stdout(Stdio::piped())
        .output()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;
    use std::fs;

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
}
