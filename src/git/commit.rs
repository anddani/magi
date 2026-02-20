use std::path::Path;

use super::git_cmd;
use crate::errors::MagiResult;

const MAX_FIXUP_COMMITS: usize = 50;

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

/// Gets a list of recent commits for fixup selection.
/// Returns a vector of strings in the format: "hash - message"
pub fn get_recent_commits_for_fixup<P: AsRef<Path>>(repo_path: P) -> MagiResult<Vec<String>> {
    let output = git_cmd(
        &repo_path,
        &[
            "log",
            &format!("-n{}", MAX_FIXUP_COMMITS),
            "--format=%h - %s",
            "HEAD",
        ],
    )
    .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(git2::Error::from_str(&format!("git log failed: {}", stderr)).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits: Vec<String> = stdout.lines().map(|line| line.to_string()).collect();

    Ok(commits)
}

/// Runs `git commit --fixup=<commit_hash> --no-edit` to create a fixup commit.
pub fn run_fixup_commit<P: AsRef<Path>>(
    repo_path: P,
    commit_hash: String,
) -> MagiResult<CommitResult> {
    let status = git_cmd(
        &repo_path,
        &["commit", &format!("--fixup={}", commit_hash), "--no-edit"],
    )
    .status()?;

    if status.success() {
        let log_output = git_cmd(&repo_path, &["log", "-1", "--format=%s"]).output()?;

        let commit_msg = String::from_utf8_lossy(&log_output.stdout)
            .trim()
            .to_string();

        Ok(CommitResult {
            success: true,
            message: format!("Created fixup: {}", commit_msg),
        })
    } else {
        Ok(CommitResult {
            success: false,
            message: "Fixup commit failed".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;

    #[test]
    fn test_get_recent_commits_for_fixup() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file1.txt", "content1")
            .stage_files(&["file1.txt"])
            .commit("First commit");

        test_repo
            .write_file_content("file2.txt", "content2")
            .stage_files(&["file2.txt"])
            .commit("Second commit");

        let commits = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();

        // Note: TestRepo::new() creates an initial commit, so we have 3 commits total
        assert_eq!(commits.len(), 3);
        assert!(commits[0].contains("Second commit"));
        assert!(commits[1].contains("First commit"));
    }

    #[test]
    fn test_get_recent_commits_for_fixup_initial_commit_only() {
        let test_repo = TestRepo::new();

        let commits = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();

        // TestRepo::new() creates one initial commit
        assert_eq!(commits.len(), 1);
        assert!(commits[0].contains("Initial commit"));
    }

    #[test]
    fn test_run_fixup_commit() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file1.txt", "content1")
            .stage_files(&["file1.txt"])
            .commit("First commit");

        // Get the hash of the first user commit (not the initial one)
        let commits = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
        let first_commit_line = &commits[0];
        let commit_hash = first_commit_line.split(" - ").next().unwrap();

        // Make a change and stage it
        test_repo
            .write_file_content("file1.txt", "modified content")
            .stage_files(&["file1.txt"]);

        // Create fixup commit
        let result = run_fixup_commit(test_repo.repo_path(), commit_hash.to_string()).unwrap();

        assert!(result.success);
        assert!(result.message.contains("fixup!"));

        // Verify the commit message
        let commits_after = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
        assert_eq!(commits_after.len(), 3); // Initial + First commit + fixup commit
        assert!(commits_after[0].contains("fixup! First commit"));
    }

    #[test]
    fn test_run_fixup_commit_without_staged_changes() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file1.txt", "content1")
            .stage_files(&["file1.txt"])
            .commit("First commit");

        // Get the hash of the first user commit
        let commits = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
        let first_commit_line = &commits[0];
        let commit_hash = first_commit_line.split(" - ").next().unwrap();

        // Try to create fixup commit without staging changes
        let result = run_fixup_commit(test_repo.repo_path(), commit_hash.to_string()).unwrap();

        assert!(!result.success);
    }
}
