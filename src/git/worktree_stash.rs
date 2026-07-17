use std::path::Path;
use std::process::Command;

use crate::git::git_cmd;

/// Creates a stash containing only the unstaged worktree changes, keeping the
/// index intact. Git has no porcelain flag for this (`--keep-index` still
/// stores the staged changes in the stash), so the stash commits are built
/// with plumbing the way magit does it: the current index is committed as a
/// temporary base so that the stash only records the worktree-vs-index diff.
pub fn create_worktree_stash(workdir: &Path, message: &str) -> Result<(), String> {
    let unstaged = run(git_cmd(workdir, &["diff", "--name-only"]))?;
    if unstaged.is_empty() {
        return Err("No unstaged changes to save".to_string());
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
    let message = if message.is_empty() {
        format!("WIP on {branch}: {head_summary}")
    } else {
        format!("On {branch}: {message}")
    };

    // Commit the current index twice: once as the base the stash is diffed
    // against (so staged changes are excluded from it), and once as the
    // stash's index commit (which therefore records no staged changes).
    let index_tree = run(git_cmd(workdir, &["write-tree"]))?;
    let base = commit_tree(workdir, &index_tree, &["HEAD"], "pre-stash index")?;
    let index_commit = commit_tree(
        workdir,
        &index_tree,
        &[&base],
        &format!("index on {branch}: {head_summary}"),
    )?;

    // Build the worktree tree in a temporary index: start from the current
    // index and overlay the files that differ between it and the worktree.
    let git_dir = run(git_cmd(workdir, &["rev-parse", "--absolute-git-dir"]))?;
    let temp_index = Path::new(&git_dir).join("magi-worktree-stash-index");
    let result = write_worktree_tree(workdir, &index_tree, &base, &temp_index);
    let _ = std::fs::remove_file(&temp_index);
    let worktree_tree = result?;

    let stash_commit = commit_tree(workdir, &worktree_tree, &[&base, &index_commit], &message)?;
    run(git_cmd(
        workdir,
        &["stash", "store", "-m", &message, &stash_commit],
    ))?;

    // keep the index: reset the worktree files to the index state
    run(git_cmd(workdir, &["checkout", "--", "."]))?;

    Ok(())
}

/// Writes a tree of the worktree state relative to `base`, using a temporary
/// index seeded from `index_tree` so the real index is untouched.
fn write_worktree_tree(
    workdir: &Path,
    index_tree: &str,
    base: &str,
    temp_index: &Path,
) -> Result<String, String> {
    let mut read_tree = git_cmd(workdir, &["read-tree", index_tree]);
    read_tree.env("GIT_INDEX_FILE", temp_index);
    run(read_tree)?;

    let changed = run(git_cmd(workdir, &["diff", "-z", "--name-only", base]))?;
    let files: Vec<&str> = changed.split('\0').filter(|f| !f.is_empty()).collect();

    let mut args = vec!["update-index", "--add", "--remove", "--"];
    args.extend(&files);
    let mut update_index = git_cmd(workdir, &args);
    update_index.env("GIT_INDEX_FILE", temp_index);
    run(update_index)?;

    let mut write_tree = git_cmd(workdir, &["write-tree"]);
    write_tree.env("GIT_INDEX_FILE", temp_index);
    run(write_tree)
}

fn commit_tree(
    workdir: &Path,
    tree: &str,
    parents: &[&str],
    message: &str,
) -> Result<String, String> {
    let mut args = vec![
        "-c",
        "commit.gpgsign=false",
        "commit-tree",
        tree,
        "-m",
        message,
    ];
    for parent in parents {
        args.push("-p");
        args.push(parent);
    }
    run(git_cmd(workdir, &args))
}

/// Runs a git command, returning trimmed stdout on success and trimmed stderr
/// as the error otherwise.
fn run(mut cmd: Command) -> Result<String, String> {
    let output = cmd.output().map_err(|err| err.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
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

    fn file_content(test_repo: &TestRepo, file_name: &str) -> String {
        std::fs::read_to_string(test_repo.repo_path().join(file_name)).unwrap()
    }

    #[test]
    fn test_create_worktree_stash_keeps_index_and_resets_worktree() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("file.txt", "committed", "Initial commit");
        test_repo
            .write_file_content("file.txt", "staged")
            .stage_files(&["file.txt"])
            .write_file_content("file.txt", "unstaged");

        let result = create_worktree_stash(test_repo.repo_path(), "wip");

        assert_eq!(result, Ok(()));
        assert_eq!(stash_messages(&test_repo), vec!["On main: wip".to_string()]);

        // The staged change is kept, in both the index and the worktree
        let staged = run(git_cmd(test_repo.repo_path(), &["show", ":file.txt"])).unwrap();
        assert_eq!(staged, "staged");
        assert_eq!(file_content(&test_repo, "file.txt"), "staged");
    }

    #[test]
    fn test_worktree_stash_contains_only_unstaged_changes() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("file.txt", "committed", "Initial commit");
        test_repo
            .write_file_content("file.txt", "staged")
            .stage_files(&["file.txt"])
            .write_file_content("file.txt", "unstaged");

        create_worktree_stash(test_repo.repo_path(), "wip").unwrap();

        // Applying the stash on top of the kept index restores only the
        // unstaged change — no duplicate of the staged change, no conflict
        run(git_cmd(test_repo.repo_path(), &["stash", "apply"])).unwrap();
        assert_eq!(file_content(&test_repo, "file.txt"), "unstaged");
        let staged = run(git_cmd(test_repo.repo_path(), &["show", ":file.txt"])).unwrap();
        assert_eq!(staged, "staged");
    }

    #[test]
    fn test_create_worktree_stash_default_message() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("file.txt", "committed", "Initial commit");
        test_repo.write_file_content("file.txt", "unstaged");

        let result = create_worktree_stash(test_repo.repo_path(), "");

        assert_eq!(result, Ok(()));
        let messages = stash_messages(&test_repo);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].starts_with("WIP on main:"));
    }

    #[test]
    fn test_create_worktree_stash_nothing_unstaged() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("file.txt", "committed", "Initial commit");
        test_repo
            .write_file_content("file.txt", "staged")
            .stage_files(&["file.txt"]);

        let result = create_worktree_stash(test_repo.repo_path(), "wip");

        assert_eq!(result, Err("No unstaged changes to save".to_string()));
        assert!(stash_messages(&test_repo).is_empty());
    }
}
