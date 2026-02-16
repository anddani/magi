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
pub fn run_amend_commit_with_editor<P: AsRef<Path>>(repo_path: P) -> MagiResult<CommitResult> {
    let status = git_cmd(&repo_path, &["commit", "--amend"]).status()?;

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

/// Runs `git commit --amend --only` to reword the last commit message
/// without including any staged changes.
/// Opens the user's configured editor with the previous commit message.
pub fn run_reword_commit_with_editor<P: AsRef<Path>>(repo_path: P) -> MagiResult<CommitResult> {
    let status = git_cmd(&repo_path, &["commit", "--amend", "--only"]).status()?;

    if status.success() {
        let log_output = git_cmd(&repo_path, &["log", "-1", "--format=%s"]).output()?;

        let commit_msg = String::from_utf8_lossy(&log_output.stdout)
            .trim()
            .to_string();

        Ok(CommitResult {
            success: true,
            message: format!("Reworded: {}", commit_msg),
        })
    } else {
        Ok(CommitResult {
            success: false,
            message: "Reword aborted".to_string(),
        })
    }
}
