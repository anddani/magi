use std::path::Path;

use super::git_cmd;
use crate::errors::MagiResult;

/// Result of a commit operation
pub struct CommitResult {
    pub success: bool,
    pub message: String,
}

/// Runs `git commit` which opens the user's configured editor.
/// This uses the git command directly (not git2) to ensure hooks run properly.
///
/// Returns a CommitResult indicating success/failure and a message.
///
/// * `flags` - list of flags. e.g. `["--all", "--no-verify"]`
pub fn run_commit_with_editor<P: AsRef<Path>>(
    repo_path: P,
    flags: Vec<String>,
) -> MagiResult<CommitResult> {
    // Using `git commit` directly ensures:
    // - Pre-commit hooks run
    // - Commit-msg hooks run
    // - Post-commit hooks run
    // - User's configured editor opens correctly
    let status = git_cmd(&repo_path, &["commit"]).args(flags).status()?;

    if status.success() {
        // Get the commit message from the last commit
        let log_output = git_cmd(&repo_path, &["log", "-1", "--format=%s"]).output()?;

        let commit_msg = String::from_utf8_lossy(&log_output.stdout)
            .trim()
            .to_string();

        Ok(CommitResult {
            success: true,
            message: commit_msg,
        })
    } else {
        // User aborted or hook failed
        // Try to get more info about what happened
        Ok(CommitResult {
            success: false,
            message: "Commit aborted".to_string(),
        })
    }
}

/// Runs `git commit --amend` to amend the last commit.
/// Opens the user's configured editor with the previous commit message.
pub fn run_amend_commit_with_editor<P: AsRef<Path>>(
    repo_path: P,
    flags: Vec<String>,
) -> MagiResult<CommitResult> {
    let status = git_cmd(&repo_path, &["commit", "--amend"])
        .args(flags)
        .status()?;

    if status.success() {
        let log_output = git_cmd(&repo_path, &["log", "-1", "--format=%s"]).output()?;

        let commit_msg = String::from_utf8_lossy(&log_output.stdout)
            .trim()
            .to_string();

        Ok(CommitResult {
            success: true,
            message: format!("Amended: {}", commit_msg),
        })
    } else {
        Ok(CommitResult {
            success: false,
            message: "Amend aborted".to_string(),
        })
    }
}

/// Runs `git commit --fixup=<commit_hash> --no-edit` to create a fixup commit.
pub fn run_fixup_commit<P: AsRef<Path>>(
    repo_path: P,
    commit_hash: String,
) -> MagiResult<CommitResult> {
    let output = git_cmd(
        &repo_path,
        &["commit", &format!("--fixup={}", commit_hash), "--no-edit"],
    )
    .output()?;

    if output.status.success() {
        let log_output = git_cmd(&repo_path, &["log", "-1", "--format=%s"]).output()?;

        let commit_msg = String::from_utf8_lossy(&log_output.stdout)
            .trim()
            .to_string();

        Ok(CommitResult {
            success: true,
            message: format!("Created fixup: {}", commit_msg),
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(CommitResult {
            success: false,
            message: format!("Fixup commit failed: {}", stderr.trim()),
        })
    }
}

/// Runs `git commit --squash=<commit_hash> --no-edit` to create a squash commit.
pub fn run_squash_commit<P: AsRef<Path>>(
    repo_path: P,
    commit_hash: String,
) -> MagiResult<CommitResult> {
    let output = git_cmd(
        &repo_path,
        &["commit", &format!("--squash={}", commit_hash), "--no-edit"],
    )
    .output()?;

    if output.status.success() {
        let log_output = git_cmd(&repo_path, &["log", "-1", "--format=%s"]).output()?;

        let commit_msg = String::from_utf8_lossy(&log_output.stdout)
            .trim()
            .to_string();

        Ok(CommitResult {
            success: true,
            message: format!("Created squash: {}", commit_msg),
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(CommitResult {
            success: false,
            message: format!("Squash commit failed: {}", stderr.trim()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;

    /// Helper to get log entries for testing (filters out graph-only entries)
    fn get_log_entries_for_test(test_repo: &TestRepo) -> Vec<crate::model::LogEntry> {
        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        let mut entries =
            super::super::log::get_log_entries(&repo, crate::msg::LogType::Current).unwrap();
        entries.retain(|e| e.is_commit());
        entries
    }

    #[test]
    fn test_run_fixup_commit() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file1.txt", "content1")
            .stage_files(&["file1.txt"])
            .commit("First commit");

        // Get the hash of the first user commit (not the initial one)
        let commits = get_log_entries_for_test(&test_repo);
        let commit_hash = commits[0].hash.as_ref().unwrap();

        // Make a change and stage it
        test_repo
            .write_file_content("file1.txt", "modified content")
            .stage_files(&["file1.txt"]);

        // Create fixup commit
        let result = run_fixup_commit(test_repo.repo_path(), commit_hash.to_string()).unwrap();

        assert!(result.success);
        assert!(result.message.contains("fixup!"));

        // Verify the commit message
        let commits_after = get_log_entries_for_test(&test_repo);
        assert_eq!(commits_after.len(), 3); // Initial + First commit + fixup commit
        assert_eq!(
            commits_after[0].message.as_deref(),
            Some("fixup! First commit")
        );
    }

    #[test]
    fn test_run_fixup_commit_without_staged_changes() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file1.txt", "content1")
            .stage_files(&["file1.txt"])
            .commit("First commit");

        // Get the hash of the first user commit
        let commits = get_log_entries_for_test(&test_repo);
        let commit_hash = commits[0].hash.as_ref().unwrap();

        // Try to create fixup commit without staging changes
        let result = run_fixup_commit(test_repo.repo_path(), commit_hash.to_string()).unwrap();

        assert!(!result.success);
    }

    #[test]
    fn test_run_squash_commit() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file1.txt", "content1")
            .stage_files(&["file1.txt"])
            .commit("First commit");

        // Get the hash of the first user commit (not the initial one)
        let commits = get_log_entries_for_test(&test_repo);
        let commit_hash = commits[0].hash.as_ref().unwrap();

        // Make a change and stage it
        test_repo
            .write_file_content("file1.txt", "modified content")
            .stage_files(&["file1.txt"]);

        // Create squash commit
        let result = run_squash_commit(test_repo.repo_path(), commit_hash.to_string()).unwrap();

        assert!(result.success);
        assert!(result.message.contains("squash!"));

        // Verify the commit message
        let commits_after = get_log_entries_for_test(&test_repo);
        assert_eq!(commits_after.len(), 3); // Initial + First commit + squash commit
        assert_eq!(
            commits_after[0].message.as_deref(),
            Some("squash! First commit")
        );
    }

    #[test]
    fn test_run_squash_commit_without_staged_changes() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file1.txt", "content1")
            .stage_files(&["file1.txt"])
            .commit("First commit");

        // Get the hash of the first user commit
        let commits = get_log_entries_for_test(&test_repo);
        let commit_hash = commits[0].hash.as_ref().unwrap();

        // Try to create squash commit without staging changes
        let result = run_squash_commit(test_repo.repo_path(), commit_hash.to_string()).unwrap();

        assert!(!result.success);
    }
}
