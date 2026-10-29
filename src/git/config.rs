use std::path::Path;

use git2::Repository;

use crate::errors::MagiResult;
use crate::git::git_cmd;

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

/// Reads a local (`--local`) git config value by arbitrary key, e.g.
/// `branch.main.rebase` or `pull.rebase`. Returns `None` when unset or empty.
/// Uses `git config` directly so any key (incl. unknown-to-libgit2 ones) works
/// and only the repository-local scope is consulted — matching magit's
/// per-repo "Configure" semantics.
pub fn get_local_config<P: AsRef<Path>>(repo_path: P, key: &str) -> Option<String> {
    let output = git_cmd(&repo_path, &["config", "--local", "--get", key])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

/// Sets a local git config `key` to `value`.
pub fn set_local_config<P: AsRef<Path>>(repo_path: P, key: &str, value: &str) -> MagiResult<()> {
    let output = git_cmd(&repo_path, &["config", "--local", key, value]).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(git2::Error::from_str(&format!("git config failed: {}", stderr)).into());
    }
    Ok(())
}

/// Unsets a local git config `key`. Tolerates the key already being unset
/// (`git config --unset` exits 5 in that case).
pub fn unset_local_config<P: AsRef<Path>>(repo_path: P, key: &str) -> MagiResult<()> {
    let output = git_cmd(&repo_path, &["config", "--local", "--unset", key]).output()?;
    // Exit code 5 = "you try to unset an option which does not exist" — not an error here.
    if !output.status.success() && output.status.code() != Some(5) {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(
            git2::Error::from_str(&format!("git config --unset failed: {}", stderr)).into(),
        );
    }
    Ok(())
}

/// Sets the upstream of `branch` to `upstream` (e.g. "origin/main") via
/// `git branch --set-upstream-to`, which writes both `branch.<b>.remote` and
/// `branch.<b>.merge`. Works for any branch, not just the checked-out one.
pub fn set_branch_upstream<P: AsRef<Path>>(
    repo_path: P,
    branch: &str,
    upstream: &str,
) -> MagiResult<()> {
    let output = git_cmd(
        &repo_path,
        &["branch", &format!("--set-upstream-to={}", upstream), branch],
    )
    .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(git2::Error::from_str(&format!("git branch failed: {}", stderr)).into());
    }
    Ok(())
}

/// Unsets the upstream of `branch` via `git branch --unset-upstream`. Tolerates
/// the branch having no upstream configured.
pub fn unset_branch_upstream<P: AsRef<Path>>(repo_path: P, branch: &str) -> MagiResult<()> {
    // Ignore failures: `--unset-upstream` errors when none is set, which is fine.
    let _ = git_cmd(&repo_path, &["branch", "--unset-upstream", branch]).output()?;
    Ok(())
}

/// Cycles a config value through `choices` the way magit's switch infixes do:
/// `None → choices[0] → choices[1] → … → last → None`. A current value that
/// isn't among `choices` resets to `None` (so the next press starts the cycle).
pub fn cycle_config_value(current: Option<&str>, choices: &[&str]) -> Option<String> {
    match current {
        None => choices.first().map(|s| s.to_string()),
        Some(value) => match choices.iter().position(|c| *c == value) {
            Some(i) => choices.get(i + 1).map(|s| s.to_string()),
            None => None,
        },
    }
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

    // ── Local config helpers (git_cmd based) ────────────────────────────────

    fn workdir(test_repo: &TestRepo) -> &std::path::Path {
        test_repo.repo.workdir().unwrap()
    }

    #[test]
    fn test_get_local_config_missing_returns_none() {
        let test_repo = TestRepo::new();
        assert_eq!(
            get_local_config(workdir(&test_repo), "branch.main.rebase"),
            None
        );
    }

    #[test]
    fn test_set_and_get_local_config_round_trip() {
        let test_repo = TestRepo::new();
        set_local_config(workdir(&test_repo), "branch.main.rebase", "true").unwrap();
        assert_eq!(
            get_local_config(workdir(&test_repo), "branch.main.rebase"),
            Some("true".to_string())
        );
    }

    #[test]
    fn test_unset_local_config_removes_value() {
        let test_repo = TestRepo::new();
        set_local_config(workdir(&test_repo), "pull.rebase", "false").unwrap();
        unset_local_config(workdir(&test_repo), "pull.rebase").unwrap();
        assert_eq!(get_local_config(workdir(&test_repo), "pull.rebase"), None);
    }

    #[test]
    fn test_unset_local_config_tolerates_missing() {
        let test_repo = TestRepo::new();
        // Unsetting a key that was never set must not error (exit code 5).
        unset_local_config(workdir(&test_repo), "branch.main.description").unwrap();
    }

    #[test]
    fn test_set_branch_upstream_writes_remote_and_merge() {
        let test_repo = TestRepo::new();
        // Create a second local branch to track; `--set-upstream-to` accepts a
        // local branch, writing `branch.<b>.remote=.` and the merge ref.
        let head = test_repo.repo.head().unwrap().peel_to_commit().unwrap();
        test_repo.repo.branch("other", &head, false).unwrap();

        set_branch_upstream(workdir(&test_repo), "main", "other").unwrap();
        assert_eq!(
            get_local_config(workdir(&test_repo), "branch.main.remote"),
            Some(".".to_string())
        );
        assert_eq!(
            get_local_config(workdir(&test_repo), "branch.main.merge"),
            Some("refs/heads/other".to_string())
        );

        unset_branch_upstream(workdir(&test_repo), "main").unwrap();
        assert_eq!(
            get_local_config(workdir(&test_repo), "branch.main.remote"),
            None
        );
        assert_eq!(
            get_local_config(workdir(&test_repo), "branch.main.merge"),
            None
        );
    }

    #[test]
    fn test_cycle_config_value_full_cycle() {
        let choices = ["true", "false"];
        // unset → first
        assert_eq!(cycle_config_value(None, &choices), Some("true".to_string()));
        // first → next
        assert_eq!(
            cycle_config_value(Some("true"), &choices),
            Some("false".to_string())
        );
        // last → unset
        assert_eq!(cycle_config_value(Some("false"), &choices), None);
        // unknown value → unset (restart next press)
        assert_eq!(cycle_config_value(Some("bogus"), &choices), None);
    }

    #[test]
    fn test_cycle_config_value_empty_choices() {
        assert_eq!(cycle_config_value(None, &[]), None);
    }
}
