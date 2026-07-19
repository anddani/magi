use std::path::Path;

use crate::git::{
    git_cmd, read_commit_message,
    worktree_stash::{commit_tree, run, store_worktree_stash},
};

/// Creates a snapshot stash of the index and working tree without resetting
/// them. Equivalent to `git stash create` followed by `git stash store`.
pub fn create_snapshot(workdir: &Path) -> Result<(), String> {
    let output = git_cmd(workdir, &["stash", "create"])
        .output()
        .map_err(|err| err.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if sha.is_empty() {
        return Err("No local changes to save".to_string());
    }

    let message = read_commit_message(workdir, &sha).unwrap_or_else(|| "Snapshot".to_string());

    let output = git_cmd(workdir, &["stash", "store", "-m", &message, &sha])
        .output()
        .map_err(|err| err.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(())
}

/// Creates a snapshot stash of the index only, keeping the index and working
/// tree intact. `git stash create` has no index-only mode, so the stash
/// commits are built with plumbing the way magit does it: the index tree is
/// committed both as the stash's index commit and as the stash commit itself.
pub fn create_index_snapshot(workdir: &Path) -> Result<(), String> {
    let staged = run(git_cmd(workdir, &["diff", "--cached", "--name-only"]))?;
    if staged.is_empty() {
        return Err("No staged changes to save".to_string());
    }
    run(git_cmd(workdir, &["rev-parse", "--verify", "HEAD"]))
        .map_err(|_| "You do not have the initial commit yet".to_string())?;

    let branch =
        run(git_cmd(workdir, &["symbolic-ref", "--short", "-q", "HEAD"])).unwrap_or_default();
    let branch = if branch.is_empty() {
        "(no branch)".to_string()
    } else {
        branch
    };
    let head_summary = run(git_cmd(workdir, &["log", "-1", "--format=%h %s", "HEAD"]))?;
    let summary = format!("{branch}: {head_summary}");

    let index_tree = run(git_cmd(workdir, &["write-tree"]))?;
    let index_commit = commit_tree(
        workdir,
        &index_tree,
        &["HEAD"],
        &format!("index on {summary}"),
    )?;
    let message = format!("WIP on {summary}");
    let stash_commit = commit_tree(workdir, &index_tree, &["HEAD", &index_commit], &message)?;

    run(git_cmd(
        workdir,
        &["stash", "store", "-m", &message, &stash_commit],
    ))?;

    Ok(())
}

/// Creates a snapshot stash of only the unstaged working tree changes,
/// keeping the index and working tree intact. Same plumbing as the
/// worktree-only stash, just without resetting the working tree afterwards.
pub fn create_worktree_snapshot(workdir: &Path) -> Result<(), String> {
    store_worktree_stash(workdir, "")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;

    fn stash_messages(test_repo: &TestRepo) -> Vec<String> {
        let output = git_cmd(test_repo.repo_path(), &["stash", "list", "--format=%gs"])
            .output()
            .unwrap();
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|l| l.to_string())
            .collect()
    }

    #[test]
    fn test_create_snapshot_keeps_working_tree() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file.txt", "content")
            .stage_files(&["file.txt"])
            .commit("Initial commit");
        test_repo.write_file_content("file.txt", "modified");

        let result = create_snapshot(test_repo.repo_path());

        assert_eq!(result, Ok(()));

        // The stash exists with the "WIP on <branch>" message...
        let messages = stash_messages(&test_repo);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("WIP on"));

        // ...and the working tree still has the modification
        let content = std::fs::read_to_string(test_repo.repo_path().join("file.txt")).unwrap();
        assert_eq!(content, "modified");
    }

    #[test]
    fn test_create_snapshot_nothing_to_save() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file.txt", "content")
            .stage_files(&["file.txt"])
            .commit("Initial commit");

        let result = create_snapshot(test_repo.repo_path());

        assert_eq!(result, Err("No local changes to save".to_string()));
        assert!(stash_messages(&test_repo).is_empty());
    }

    #[test]
    fn test_create_index_snapshot_keeps_index_and_worktree() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file.txt", "content")
            .stage_files(&["file.txt"])
            .commit("Initial commit");
        test_repo
            .write_file_content("file.txt", "staged")
            .stage_files(&["file.txt"])
            .write_file_content("file.txt", "unstaged");

        let result = create_index_snapshot(test_repo.repo_path());

        assert_eq!(result, Ok(()));

        let messages = stash_messages(&test_repo);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].starts_with("WIP on"));

        // Nothing is reset: the index still holds the staged change and the
        // working tree still holds the unstaged one
        let staged = run(git_cmd(test_repo.repo_path(), &["show", ":file.txt"])).unwrap();
        assert_eq!(staged, "staged");
        let content = std::fs::read_to_string(test_repo.repo_path().join("file.txt")).unwrap();
        assert_eq!(content, "unstaged");
    }

    #[test]
    fn test_index_snapshot_contains_only_staged_changes() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file.txt", "content")
            .stage_files(&["file.txt"])
            .commit("Initial commit");
        test_repo
            .write_file_content("file.txt", "staged")
            .stage_files(&["file.txt"])
            .write_file_content("file.txt", "unstaged");

        create_index_snapshot(test_repo.repo_path()).unwrap();

        // The stash commit's tree matches the index, not the working tree
        let stashed = run(git_cmd(
            test_repo.repo_path(),
            &["show", "stash@{0}:file.txt"],
        ))
        .unwrap();
        assert_eq!(stashed, "staged");
    }

    #[test]
    fn test_create_index_snapshot_nothing_staged() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file.txt", "content")
            .stage_files(&["file.txt"])
            .commit("Initial commit");
        test_repo.write_file_content("file.txt", "unstaged");

        let result = create_index_snapshot(test_repo.repo_path());

        assert_eq!(result, Err("No staged changes to save".to_string()));
        assert!(stash_messages(&test_repo).is_empty());
    }

    #[test]
    fn test_create_worktree_snapshot_keeps_index_and_worktree() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file.txt", "content")
            .stage_files(&["file.txt"])
            .commit("Initial commit");
        test_repo
            .write_file_content("file.txt", "staged")
            .stage_files(&["file.txt"])
            .write_file_content("file.txt", "unstaged");

        let result = create_worktree_snapshot(test_repo.repo_path());

        assert_eq!(result, Ok(()));

        let messages = stash_messages(&test_repo);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].starts_with("WIP on"));

        // Nothing is reset: the index still holds the staged change and the
        // working tree still holds the unstaged one
        let staged = run(git_cmd(test_repo.repo_path(), &["show", ":file.txt"])).unwrap();
        assert_eq!(staged, "staged");
        let content = std::fs::read_to_string(test_repo.repo_path().join("file.txt")).unwrap();
        assert_eq!(content, "unstaged");
    }

    #[test]
    fn test_worktree_snapshot_contains_only_unstaged_changes() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file.txt", "content")
            .stage_files(&["file.txt"])
            .commit("Initial commit");
        test_repo
            .write_file_content("file.txt", "staged")
            .stage_files(&["file.txt"])
            .write_file_content("file.txt", "unstaged");

        create_worktree_snapshot(test_repo.repo_path()).unwrap();

        // The stash commit's tree matches the working tree, and its index
        // commit records no staged changes
        let stashed = run(git_cmd(
            test_repo.repo_path(),
            &["show", "stash@{0}:file.txt"],
        ))
        .unwrap();
        assert_eq!(stashed, "unstaged");
        let stashed_index = run(git_cmd(
            test_repo.repo_path(),
            &["diff", "--name-only", "stash@{0}^", "stash@{0}^2"],
        ))
        .unwrap();
        assert_eq!(stashed_index, "");
    }

    #[test]
    fn test_create_worktree_snapshot_nothing_unstaged() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file.txt", "content")
            .stage_files(&["file.txt"])
            .commit("Initial commit");
        test_repo
            .write_file_content("file.txt", "staged")
            .stage_files(&["file.txt"]);

        let result = create_worktree_snapshot(test_repo.repo_path());

        assert_eq!(result, Err("No unstaged changes to save".to_string()));
        assert!(stash_messages(&test_repo).is_empty());
    }
}
