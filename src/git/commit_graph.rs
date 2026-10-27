use std::path::Path;
use std::process::{Command, Stdio};

use git2::Repository;

use super::git_cmd;

/// Returns true if the repository already has a commit-graph, either as a
/// single `commit-graph` file or as a split `commit-graphs/` chain.
pub fn commit_graph_exists(repository: &Repository) -> bool {
    // Objects (and thus the commit-graph) live in the common dir, which differs
    // from the git dir for linked worktrees.
    let info_dir = repository.commondir().join("objects").join("info");
    info_dir.join("commit-graph").exists()
        || info_dir
            .join("commit-graphs")
            .join("commit-graph-chain")
            .exists()
}

/// Builds the command that writes the commit-graph for the repository at
/// `workdir`. Kept separate so tests can run it synchronously.
fn write_command(workdir: &Path) -> Command {
    let mut cmd = git_cmd(workdir, &["commit-graph", "write", "--reachable"]);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    cmd
}

/// If the repository has no commit-graph, spawn a background `git commit-graph
/// write` to create one.
///
/// `git log --graph` implies `--topo-order`, which cannot be done incrementally:
/// git must walk the *entire* reachable history before emitting the first line,
/// so even `-n256` stays slow (≈7s on nixpkgs' 1.1M commits). The commit-graph
/// stores generation numbers that let topo-order run incrementally, taking the
/// same query to ≈0.05s — a ~150x speedup that also benefits every other
/// revwalk. The file is a pure cache that git itself writes during `gc`/`fetch`
/// when configured.
///
/// The write runs detached so it never blocks the UI; the first log view in a
/// fresh session may still be slow, but once the write finishes the file is
/// picked up by subsequent git invocations. Does nothing when a commit-graph
/// already exists, so the cost is paid at most once per repository.
pub fn ensure_commit_graph(repository: &Repository, workdir: &Path) {
    if commit_graph_exists(repository) {
        return;
    }
    // Best effort: if git is missing or the spawn fails, magi still works, just
    // without the speedup.
    let _ = write_command(workdir).spawn();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;

    #[test]
    fn test_commit_graph_absent_on_fresh_repo() {
        let test_repo = TestRepo::new();
        assert!(!commit_graph_exists(&test_repo.repo));
    }

    #[test]
    fn test_write_command_creates_commit_graph() {
        let test_repo = TestRepo::new();
        assert!(!commit_graph_exists(&test_repo.repo));

        // Run the write synchronously (the production path spawns it detached).
        let status = write_command(test_repo.repo_path())
            .status()
            .expect("git commit-graph write should run");
        assert!(status.success());

        assert!(commit_graph_exists(&test_repo.repo));
    }

    #[test]
    fn test_ensure_commit_graph_is_noop_when_present() {
        let test_repo = TestRepo::new();
        write_command(test_repo.repo_path())
            .status()
            .expect("git commit-graph write should run");
        assert!(commit_graph_exists(&test_repo.repo));

        // Should return immediately without error when one already exists.
        ensure_commit_graph(&test_repo.repo, test_repo.repo_path());
        assert!(commit_graph_exists(&test_repo.repo));
    }
}
