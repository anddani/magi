use std::path::Path;
use std::process::{Command, Stdio};

use git2::Repository;

use crate::errors::MagiResult;

/// Gets the list of configured remotes.
/// Returns an empty vec if no remotes are configured.
pub fn get_remotes(repo: &Repository) -> Vec<String> {
    repo.remotes()
        .map(|remotes| remotes.iter().flatten().map(|s| s.to_string()).collect())
        .unwrap_or_default()
}

/// Gets the current branch name.
pub fn get_current_branch<P: AsRef<Path>>(repo_path: P) -> MagiResult<Option<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path.as_ref())
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if branch == "HEAD" {
            // Detached HEAD
            Ok(None)
        } else {
            Ok(Some(branch))
        }
    } else {
        Ok(None)
    }
}

/// Gets the upstream branch name for the current branch.
/// Returns None if no upstream is configured.
pub fn get_upstream_branch<P: AsRef<Path>>(repo_path: P) -> MagiResult<Option<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path.as_ref())
        .args(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        let upstream = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if upstream.is_empty() {
            Ok(None)
        } else {
            Ok(Some(upstream))
        }
    } else {
        // No upstream configured
        Ok(None)
    }
}
