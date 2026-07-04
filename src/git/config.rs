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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;

    fn set_config(test_repo: &TestRepo, key: &str, value: &str) {
        test_repo
            .repo
            .config()
            .unwrap()
            .set_str(key, value)
            .unwrap();
    }

    /// `repo.config()` also reads the developer's global/system config, so
    /// tests that rely on `remote.pushDefault` being absent shadow it with an
    /// empty repo-local value instead (which the functions treat as unset).
    fn clear_push_default(test_repo: &TestRepo) {
        set_config(test_repo, "remote.pushDefault", "");
    }

    #[test]
    fn test_get_remote_url_missing_returns_none() {
        let test_repo = TestRepo::new();
        assert_eq!(get_remote_url(&test_repo.repo, "nonexistent-remote"), None);
    }

    #[test]
    fn test_get_remote_url_returns_configured_url() {
        let test_repo = TestRepo::new();
        set_config(
            &test_repo,
            "remote.origin.url",
            "git@example.com:user/repo.git",
        );

        assert_eq!(
            get_remote_url(&test_repo.repo, "origin"),
            Some("git@example.com:user/repo.git".to_string())
        );
    }

    #[test]
    fn test_get_remote_url_empty_returns_none() {
        let test_repo = TestRepo::new();
        set_config(&test_repo, "remote.origin.url", "");

        assert_eq!(get_remote_url(&test_repo.repo, "origin"), None);
    }

    #[test]
    fn test_get_upstream_remote_missing_returns_none() {
        let test_repo = TestRepo::new();
        assert_eq!(get_upstream_remote(&test_repo.repo, "feature-xyz"), None);
    }

    #[test]
    fn test_get_upstream_remote_returns_configured_remote() {
        let test_repo = TestRepo::new();
        set_config(&test_repo, "branch.main.remote", "origin");

        assert_eq!(
            get_upstream_remote(&test_repo.repo, "main"),
            Some("origin".to_string())
        );
    }

    #[test]
    fn test_get_upstream_remote_empty_returns_none() {
        let test_repo = TestRepo::new();
        set_config(&test_repo, "branch.main.remote", "");

        assert_eq!(get_upstream_remote(&test_repo.repo, "main"), None);
    }

    #[test]
    fn test_get_push_remote_missing_returns_none() {
        let test_repo = TestRepo::new();
        clear_push_default(&test_repo);

        assert_eq!(get_push_remote(&test_repo.repo, "main"), None);
    }

    #[test]
    fn test_get_push_remote_falls_back_to_push_default() {
        let test_repo = TestRepo::new();
        set_config(&test_repo, "remote.pushDefault", "upstream");

        assert_eq!(
            get_push_remote(&test_repo.repo, "main"),
            Some("upstream".to_string())
        );
    }

    #[test]
    fn test_get_push_remote_prefers_branch_push_remote_over_default() {
        let test_repo = TestRepo::new();
        set_config(&test_repo, "remote.pushDefault", "upstream");
        set_config(&test_repo, "branch.main.pushRemote", "fork");

        assert_eq!(
            get_push_remote(&test_repo.repo, "main"),
            Some("fork".to_string())
        );
        // Another branch without a pushRemote still gets the default.
        assert_eq!(
            get_push_remote(&test_repo.repo, "other"),
            Some("upstream".to_string())
        );
    }

    #[test]
    fn test_set_push_remote_round_trip() {
        let test_repo = TestRepo::new();
        clear_push_default(&test_repo);

        set_push_remote(&test_repo.repo, "main", "myremote").unwrap();

        // Readable through the module's own getter...
        assert_eq!(
            get_push_remote(&test_repo.repo, "main"),
            Some("myremote".to_string())
        );
        // ...and stored under the expected raw key.
        assert_eq!(
            test_repo
                .repo
                .config()
                .unwrap()
                .get_string("branch.main.pushRemote")
                .unwrap(),
            "myremote"
        );
    }

    #[test]
    fn test_set_push_remote_overwrites_existing_value() {
        let test_repo = TestRepo::new();

        set_push_remote(&test_repo.repo, "main", "first").unwrap();
        set_push_remote(&test_repo.repo, "main", "second").unwrap();

        assert_eq!(
            get_push_remote(&test_repo.repo, "main"),
            Some("second".to_string())
        );
    }

    #[test]
    fn test_get_remote_defaults_to_origin() {
        let test_repo = TestRepo::new();
        clear_push_default(&test_repo);

        assert_eq!(get_remote(&test_repo.repo, "main"), "origin");
    }

    #[test]
    fn test_get_remote_falls_back_to_upstream_remote() {
        let test_repo = TestRepo::new();
        clear_push_default(&test_repo);
        set_config(&test_repo, "branch.main.remote", "upstream");

        assert_eq!(get_remote(&test_repo.repo, "main"), "upstream");
    }

    #[test]
    fn test_get_remote_prefers_push_remote_over_upstream() {
        let test_repo = TestRepo::new();
        set_config(&test_repo, "branch.main.remote", "upstream");
        set_config(&test_repo, "branch.main.pushRemote", "fork");

        assert_eq!(get_remote(&test_repo.repo, "main"), "fork");
    }
}
