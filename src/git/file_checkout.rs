use std::path::Path;
use std::process::Stdio;

use git2::Repository;

use super::git_cmd;
use crate::errors::MagiResult;

/// Returns the list of all tracked files in the repository index.
pub fn get_tracked_files(repo: &Repository) -> Vec<String> {
    repo.index()
        .map(|index| {
            index
                .iter()
                .map(|entry| String::from_utf8_lossy(&entry.path).into_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// Restore a single file to its state at the given revision.
/// Equivalent to `git checkout <revision> -- <file>`.
pub fn file_checkout<P: AsRef<Path>>(
    repo_path: P,
    revision: &str,
    file: &str,
) -> MagiResult<Result<(), String>> {
    let output = git_cmd(&repo_path, &["checkout", revision, "--", file])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        Ok(Ok(()))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Ok(Err(if stderr.is_empty() {
            format!("Failed to checkout {} from {}", file, revision)
        } else {
            stderr
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;

    #[test]
    fn test_get_tracked_files_returns_committed_files() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("tracked.txt", "hello")
            .stage_files(&["tracked.txt"])
            .commit("Add tracked file");

        let files = get_tracked_files(&test_repo.repo);
        assert!(files.iter().any(|f| f == "tracked.txt"));
    }

    #[test]
    fn test_get_tracked_files_empty_for_bare_repo() {
        // Fresh TestRepo with no staged/committed files has only "initial.txt" from TestRepo::new()
        let test_repo = TestRepo::new();
        let files = get_tracked_files(&test_repo.repo);
        // TestRepo::new() commits an initial file
        assert!(!files.is_empty());
    }

    #[test]
    fn test_file_checkout_restores_file_content() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();

        // Commit "original" content
        test_repo
            .write_file_content("restore.txt", "original content")
            .stage_files(&["restore.txt"])
            .commit("Original commit");

        // Record the commit hash
        let original_hash = {
            let repo = git2::Repository::open(repo_path).unwrap();
            repo.head()
                .unwrap()
                .peel_to_commit()
                .unwrap()
                .id()
                .to_string()
        };

        // Overwrite the file and commit again
        test_repo
            .write_file_content("restore.txt", "modified content")
            .stage_files(&["restore.txt"])
            .commit("Modified commit");

        // Now restore the file from the original commit
        let result = file_checkout(repo_path, &original_hash, "restore.txt").unwrap();
        assert!(result.is_ok(), "file_checkout should succeed");

        // Verify file content was restored
        let content = std::fs::read_to_string(repo_path.join("restore.txt")).unwrap();
        assert_eq!(content.trim(), "original content");
    }

    #[test]
    fn test_file_checkout_invalid_revision_returns_error() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();

        let result = file_checkout(repo_path, "nonexistent-revision-xyz", "initial.txt").unwrap();
        assert!(result.is_err(), "should fail for invalid revision");
    }

    #[test]
    fn test_file_checkout_invalid_file_returns_error() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();

        let result = file_checkout(repo_path, "HEAD", "nonexistent-file.txt").unwrap();
        assert!(result.is_err(), "should fail for nonexistent file");
    }
}
