use std::{
    path::Path,
    process::{Command, ExitStatus, Stdio},
};

use super::git_cmd;
use crate::errors::MagiResult;

/// Result of a commit operation
pub struct CommitResult {
    pub success: bool,
    pub message: String,
}

pub fn get_commit_result<P: AsRef<Path>>(
    repo_path: P,
    status: ExitStatus,
    op: &str,
) -> MagiResult<CommitResult> {
    get_commit_result_with_stderr(repo_path, status, "", op)
}

pub fn get_commit_result_with_stderr<P: AsRef<Path>>(
    repo_path: P,
    status: ExitStatus,
    stderr: &str,
    op: &str,
) -> MagiResult<CommitResult> {
    if status.success() {
        let log_output = git_cmd(&repo_path, &["log", "-1", "--format=%s"]).output()?;

        let commit_msg = String::from_utf8_lossy(&log_output.stdout)
            .trim()
            .to_string();

        Ok(CommitResult {
            success: true,
            message: format!("{}: {}", op, commit_msg),
        })
    } else {
        Ok(CommitResult {
            success: false,
            message: abort_message(op, stderr),
        })
    }
}

/// Builds the failure message for an aborted commit, including the most
/// useful line of git's stderr: the first `error:`/`fatal:` line if present,
/// otherwise the first non-empty line.
fn abort_message(op: &str, stderr: &str) -> String {
    let lines = || stderr.lines().map(str::trim).filter(|l| !l.is_empty());
    let reason = lines()
        .find(|l| l.starts_with("error:") || l.starts_with("fatal:"))
        .map(|l| {
            l.trim_start_matches("error:")
                .trim_start_matches("fatal:")
                .trim()
                .trim_end_matches(':')
        })
        .or_else(|| lines().next());
    match reason {
        Some(reason) => format!("{} aborted: {}", op, reason),
        None => format!("{} aborted", op),
    }
}

/// Runs a git command with stdin/stdout attached to the terminal (so the
/// user's editor works) while capturing stderr for error reporting.
fn status_capturing_stderr(cmd: &mut Command) -> std::io::Result<(ExitStatus, String)> {
    let child = cmd.stderr(Stdio::piped()).spawn()?;
    let output = child.wait_with_output()?;
    Ok((
        output.status,
        String::from_utf8_lossy(&output.stderr).to_string(),
    ))
}

/// Checks whether git can create a signed commit with the current
/// configuration. Returns git's stderr when signing fails, `None` when it
/// works. The probe signs a dangling commit of the empty tree, so the index,
/// worktree and HEAD are untouched (git gc eventually removes the object).
pub fn signing_error<P: AsRef<Path>>(repo_path: P) -> MagiResult<Option<String>> {
    let tree = git_cmd(&repo_path, &["hash-object", "-w", "-t", "tree", "--stdin"])
        .stdin(Stdio::null())
        .output()?;
    if !tree.status.success() {
        return Ok(Some(
            String::from_utf8_lossy(&tree.stderr).trim().to_string(),
        ));
    }
    let tree_hash = String::from_utf8_lossy(&tree.stdout).trim().to_string();

    let probe = git_cmd(
        &repo_path,
        &["commit-tree", &tree_hash, "-S", "-m", "magi signing check"],
    )
    .stdin(Stdio::null())
    .output()?;
    if probe.status.success() {
        Ok(None)
    } else {
        // Drop gpg's machine-readable status lines; they add no information
        // for the user.
        let message = String::from_utf8_lossy(&probe.stderr)
            .lines()
            .filter(|l| !l.trim_start().starts_with("[GNUPG:]"))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();
        Ok(Some(message))
    }
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
    let (status, stderr) = status_capturing_stderr(git_cmd(&repo_path, &["commit"]).args(flags))?;

    get_commit_result_with_stderr(repo_path, status, &stderr, "Commit")
}

/// Lists authors from the commit history as `Name <email>` strings,
/// deduplicated and ordered from most recent commit to oldest.
pub fn list_authors<P: AsRef<Path>>(repo_path: P) -> MagiResult<Vec<String>> {
    let output = git_cmd(&repo_path, &["log", "-n9999", "--format=%aN <%aE>"]).output()?;

    let mut seen = std::collections::HashSet::new();
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| seen.insert(line.to_string()))
        .map(str::to_string)
        .collect())
}

/// Runs `git commit --amend` to amend the last commit.
/// Opens the user's configured editor with the previous commit message.
pub fn run_amend_commit_with_editor<P: AsRef<Path>>(
    repo_path: P,
    flags: Vec<String>,
) -> MagiResult<CommitResult> {
    let (status, stderr) =
        status_capturing_stderr(git_cmd(&repo_path, &["commit", "--amend"]).args(flags))?;

    get_commit_result_with_stderr(repo_path, status, &stderr, "Amend")
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

    get_commit_result_with_stderr(
        repo_path,
        output.status,
        &String::from_utf8_lossy(&output.stderr),
        "Fixup",
    )
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

    get_commit_result_with_stderr(
        repo_path,
        output.status,
        &String::from_utf8_lossy(&output.stderr),
        "Squash",
    )
}

/// Runs `git commit --squash=<commit_hash> --edit` to create an augment commit.
/// Opens the user's configured editor to author a temporary squash message.
pub fn run_augment_commit<P: AsRef<Path>>(
    repo_path: P,
    commit_hash: String,
) -> MagiResult<CommitResult> {
    let (status, stderr) = status_capturing_stderr(&mut git_cmd(
        &repo_path,
        &["commit", &format!("--squash={}", commit_hash), "--edit"],
    ))?;

    get_commit_result_with_stderr(repo_path, status, &stderr, "Augment")
}

/// Runs `git commit --fixup=reword:<commit_hash> --edit` to revise a commit message.
/// Opens the user's configured editor to author the revised commit message.
/// Unlike other fixup types, this does NOT require staged changes.
pub fn run_revise_commit<P: AsRef<Path>>(
    repo_path: P,
    commit_hash: String,
) -> MagiResult<CommitResult> {
    let (status, stderr) = status_capturing_stderr(&mut git_cmd(
        &repo_path,
        &[
            "commit",
            &format!("--fixup=reword:{}", commit_hash),
            "--edit",
        ],
    ))?;

    get_commit_result_with_stderr(repo_path, status, &stderr, "Revise")
}

/// Runs `git commit --fixup=amend:<commit_hash> --edit` to create an alter commit.
/// Opens the user's configured editor to author the final commit message.
pub fn run_alter_commit<P: AsRef<Path>>(
    repo_path: P,
    commit_hash: String,
) -> MagiResult<CommitResult> {
    let (status, stderr) = status_capturing_stderr(&mut git_cmd(
        &repo_path,
        &[
            "commit",
            &format!("--fixup=amend:{}", commit_hash),
            "--edit",
        ],
    ))?;

    get_commit_result_with_stderr(repo_path, status, &stderr, "Alter")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;

    /// Helper to get log entries for testing (filters out graph-only entries)
    fn get_log_entries_for_test(test_repo: &TestRepo) -> Vec<crate::model::LogEntry> {
        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        let mut entries =
            super::super::log::get_log_entries(&repo, &crate::msg::LogType::Current, true, false)
                .unwrap();
        entries.retain(|e| e.is_commit());
        entries
    }

    #[test]
    fn test_abort_message_uses_error_line_from_stderr() {
        let stderr = "hint: something\nerror: gpg failed to sign the data:\ngpg: signing failed: No secret key\nfatal: failed to write commit object\n";
        assert_eq!(
            abort_message("Commit", stderr),
            "Commit aborted: gpg failed to sign the data"
        );
    }

    #[test]
    fn test_abort_message_falls_back_to_first_line() {
        assert_eq!(
            abort_message("Commit", "Aborting commit due to empty commit message.\n"),
            "Commit aborted: Aborting commit due to empty commit message."
        );
    }

    #[test]
    fn test_abort_message_without_stderr() {
        assert_eq!(abort_message("Commit", ""), "Commit aborted");
    }

    #[test]
    fn test_signing_error_reports_gpg_failure() {
        let test_repo = TestRepo::new();
        // Point gpg at a program that always fails, like a missing key does.
        git_cmd(test_repo.repo_path(), &["config", "gpg.program", "false"])
            .output()
            .unwrap();

        let err = signing_error(test_repo.repo_path()).unwrap();

        assert!(err.is_some());
        assert!(err.unwrap().contains("gpg"));
    }

    #[test]
    fn test_signing_error_none_when_signing_works() {
        let test_repo = TestRepo::new();
        // Fake gpg that emits the SIG_CREATED status line git looks for. It
        // must drain stdin: git's pipe_command treats EPIPE on the child's
        // stdin as a signing failure, so exiting before git finishes writing
        // the payload makes the probe flaky (seen on loaded CI runners).
        let fake_gpg = test_repo.repo_path().join("fake-gpg.sh");
        std::fs::write(
            &fake_gpg,
            "#!/bin/sh\ncat >/dev/null\necho \"[GNUPG:] SIG_CREATED \" >&2\necho fake-signature\n",
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&fake_gpg, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        git_cmd(
            test_repo.repo_path(),
            &["config", "gpg.program", fake_gpg.to_str().unwrap()],
        )
        .output()
        .unwrap();

        assert!(signing_error(test_repo.repo_path()).unwrap().is_none());
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
