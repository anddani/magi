use git2::Repository;

use crate::errors::MagiResult;

/// Gets the `remote.<name>.url` for a given remote
pub fn get_remote_url(repo: &Repository, remote: &str) -> Option<String> {
    let config = repo.config().ok()?;
    if let Ok(url) = config.get_string(&format!("remote.{}.url", remote))
        && !url.is_empty()
    {
        return Some(url);
    }
    None
}

/// Gets the upstream remote for the given branch.
/// Checks `branch.<name>.remote`.
/// Returns None if not configured
pub fn get_upstream_remote(repo: &Repository, branch: &str) -> Option<String> {
    let config = repo.config().ok()?;
    if let Ok(remote) = config.get_string(&format!("branch.{}.remote", branch))
        && !remote.is_empty()
    {
        return Some(remote);
    }
    None
}

/// Gets the push remote for the given branch.
/// Checks `branch.<name>.pushRemote` first, then falls back to `remote.pushDefault`.
/// Returns None if neither is configured.
pub fn get_push_remote(repo: &Repository, branch: &str) -> Option<String> {
    let config = repo.config().ok()?;
    // Check branch-specific push remote first
    if let Ok(remote) = config.get_string(&format!("branch.{}.pushRemote", branch))
        && !remote.is_empty()
    {
        return Some(remote);
    }
    // Fall back to global push default
    if let Ok(remote) = config.get_string("remote.pushDefault")
        && !remote.is_empty()
    {
        return Some(remote);
    }
    None
}

/// Gets the push remote, if set.
/// Otherwise the upstream remote, if set.
/// Otherwise "origin"
pub fn get_remote(repo: &Repository, branch: &str) -> String {
    get_push_remote(repo, branch)
        .or_else(|| get_upstream_remote(repo, branch))
        .unwrap_or_else(|| "origin".to_string())
}

/// Sets `branch.<name>.pushRemote` for the given branch.
pub fn set_push_remote(repo: &Repository, branch: &str, remote: &str) -> MagiResult<()> {
    let mut config = repo.config()?;
    config.set_str(&format!("branch.{}.pushRemote", branch), remote)?;
    Ok(())
}
