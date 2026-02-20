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
/// Returns a vector of LogEntry objects with commit information.
pub fn get_recent_commits_for_fixup<P: AsRef<Path>>(
    repo_path: P,
) -> MagiResult<Vec<crate::model::LogEntry>> {
    const SEPARATOR: char = '\x0c'; // Form feed character

    // Format: hash<sep>refs<sep>author<sep>date<sep>message
    let format = format!(
        "%h{}%D{}%aN{}%ar{}%s",
        SEPARATOR, SEPARATOR, SEPARATOR, SEPARATOR
    );

    let output = git_cmd(
        &repo_path,
        &[
            "log",
            &format!("-n{}", MAX_FIXUP_COMMITS),
            &format!("--format={}", format),
            "--decorate=short",
            "HEAD",
        ],
    )
    .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(git2::Error::from_str(&format!("git log failed: {}", stderr)).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits = super::log::parse_log_output(&stdout, &[]);

    Ok(commits)
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
        assert_eq!(commits[0].message.as_deref(), Some("Second commit"));
        assert_eq!(commits[1].message.as_deref(), Some("First commit"));
    }

    #[test]
    fn test_get_recent_commits_for_fixup_initial_commit_only() {
        let test_repo = TestRepo::new();

        let commits = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();

        // TestRepo::new() creates one initial commit
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].message.as_deref(), Some("Initial commit"));
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
        let commits_after = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
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
        let commits = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
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
        let commits = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
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
        let commits_after = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
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
        let commits = get_recent_commits_for_fixup(test_repo.repo_path()).unwrap();
        let commit_hash = commits[0].hash.as_ref().unwrap();

        // Try to create squash commit without staging changes
        let result = run_squash_commit(test_repo.repo_path(), commit_hash.to_string()).unwrap();

        assert!(!result.success);
    }
}
