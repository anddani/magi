use std::path::Path;
use std::process::{Command, Stdio};

use crate::errors::MagiResult;

/// Result of a push operation
pub struct PushResult {
    pub success: bool,
    pub message: String,
}

/// Gets the list of configured remotes.
/// Returns an empty vec if no remotes are configured.
pub fn get_remotes<P: AsRef<Path>>(repo_path: P) -> MagiResult<Vec<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path.as_ref())
        .arg("remote")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        let remotes = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Ok(remotes)
    } else {
        Ok(vec![])
    }
}

/// Pushes to the upstream branch.
/// If upstream is set, pushes to it. Otherwise, returns an error.
pub fn push_to_upstream<P: AsRef<Path>>(repo_path: P) -> MagiResult<PushResult> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path.as_ref())
        .args(["push", "-v"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        Ok(PushResult {
            success: true,
            message: "Pushed to upstream".to_string(),
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Ok(PushResult {
            success: false,
            message: if stderr.is_empty() {
                "Push failed".to_string()
            } else {
                stderr
            },
        })
    }
}

/// Pushes to a specified remote branch, setting it as upstream.
/// This is used when no upstream is configured.
pub fn push_with_set_upstream<P: AsRef<Path>>(
    repo_path: P,
    remote: &str,
    branch: &str,
) -> MagiResult<PushResult> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path.as_ref())
        .args(["push", "-v", "--set-upstream", remote, branch])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        Ok(PushResult {
            success: true,
            message: format!("Pushed to {}/{}", remote, branch),
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Ok(PushResult {
            success: false,
            message: if stderr.is_empty() {
                "Push failed".to_string()
            } else {
                stderr
            },
        })
    }
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
