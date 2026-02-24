use std::path::Path;
use std::process::Stdio;

use git2::Repository;

use super::git_cmd;
use crate::errors::MagiResult;

/// Gets the list of local tags.
/// Returns an empty vec if no tags exist.
pub fn get_local_tags(repo: &Repository) -> Vec<String> {
    let mut tags = Vec::new();
    let _ = repo.tag_foreach(|_oid, name| {
        let name_str = String::from_utf8_lossy(name);
        if let Some(tag_name) = name_str.strip_prefix("refs/tags/") {
            tags.push(tag_name.to_string());
        }
        true
    });
    tags.sort();
    tags
}

/// Gets the list of configured remotes.
/// Returns an empty vec if no remotes are configured.
pub fn get_remotes(repo: &Repository) -> Vec<String> {
    repo.remotes()
        .map(|remotes| remotes.iter().flatten().map(|s| s.to_string()).collect())
        .unwrap_or_default()
}

/// Gets the current branch name.
pub fn get_current_branch<P: AsRef<Path>>(repo_path: P) -> MagiResult<Option<String>> {
    let output = git_cmd(&repo_path, &["rev-parse", "--abbrev-ref", "HEAD"])
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

/// Sets the upstream branch for the current branch.
/// The upstream should be in the format "remote/branch" (e.g., "origin/main").
pub fn set_upstream_branch(repo: &Repository, upstream: &str) -> MagiResult<()> {
    let head = repo.head()?;
    let branch_name = head
        .shorthand()
        .ok_or_else(|| git2::Error::from_str("Could not get branch name"))?;

    let mut branch = repo.find_branch(branch_name, git2::BranchType::Local)?;
    branch.set_upstream(Some(upstream))?;
    Ok(())
}

/// Parses a remote/branch string into its components.
/// e.g., "origin/main" -> ("origin", "main")
/// If no slash is present, assumes it's just the remote name.
pub fn parse_remote_branch(upstream: &str) -> (String, String) {
    if let Some((remote, branch)) = upstream.split_once('/') {
        (remote.to_string(), branch.to_string())
    } else {
        (upstream.to_string(), String::new())
    }
}

/// Gets the push remote for the given branch.
/// Checks `branch.<name>.pushRemote` first, then falls back to `remote.pushDefault`.
/// Returns None if neither is configured.
pub fn get_push_remote(repo: &Repository, branch: &str) -> Option<String> {
    let config = repo.config().ok()?;
    // Check branch-specific push remote first
    if let Ok(remote) = config.get_string(&format!("branch.{}.pushRemote", branch)) {
        if !remote.is_empty() {
            return Some(remote);
        }
    }
    // Fall back to global push default
    if let Ok(remote) = config.get_string("remote.pushDefault") {
        if !remote.is_empty() {
            return Some(remote);
        }
    }
    None
}

/// Sets `branch.<name>.pushRemote` for the given branch.
pub fn set_push_remote(repo: &Repository, branch: &str, remote: &str) -> MagiResult<()> {
    let mut config = repo.config()?;
    config.set_str(&format!("branch.{}.pushRemote", branch), remote)?;
    Ok(())
}

/// Gets the upstream branch name for the current branch.
/// Returns None if no upstream is configured.
pub fn get_upstream_branch<P: AsRef<Path>>(repo_path: P) -> MagiResult<Option<String>> {
    let output = git_cmd(
        &repo_path,
        &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;
    use std::process::Command as StdCommand;

    #[test]
    fn test_get_local_tags_empty_repo() {
        let test_repo = TestRepo::new();
        let tags = get_local_tags(&test_repo.repo);
        assert!(tags.is_empty());
    }

    #[test]
    fn test_get_local_tags_with_tags() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();

        // Create some tags
        StdCommand::new("git")
            .arg("-C")
            .arg(repo_path)
            .args(["tag", "v1.0.0"])
            .output()
            .expect("Failed to create tag");

        StdCommand::new("git")
            .arg("-C")
            .arg(repo_path)
            .args(["tag", "v2.0.0"])
            .output()
            .expect("Failed to create tag");

        let tags = get_local_tags(&test_repo.repo);
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"v1.0.0".to_string()));
        assert!(tags.contains(&"v2.0.0".to_string()));
    }

    #[test]
    fn test_get_local_tags_sorted() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();

        // Create tags in reverse order
        StdCommand::new("git")
            .arg("-C")
            .arg(repo_path)
            .args(["tag", "z-tag"])
            .output()
            .expect("Failed to create tag");

        StdCommand::new("git")
            .arg("-C")
            .arg(repo_path)
            .args(["tag", "a-tag"])
            .output()
            .expect("Failed to create tag");

        let tags = get_local_tags(&test_repo.repo);
        assert_eq!(tags, vec!["a-tag".to_string(), "z-tag".to_string()]);
    }
}
