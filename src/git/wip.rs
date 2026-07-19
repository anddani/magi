use std::path::Path;

use crate::git::{
    git_cmd,
    worktree_stash::{commit_tree, run},
};

const WIP_MESSAGE: &str = "wip-save tracked files";

/// Commits the current index and working tree states of all tracked files to
/// the work-in-progress refs (`refs/wip/index/<ref>` and
/// `refs/wip/wtree/<ref>`) without touching the index, the working tree or
/// the current branch. Equivalent to `magit-wip-commit`.
pub fn commit_to_wip_refs(workdir: &Path) -> Result<(), String> {
    let head_ref =
        run(git_cmd(workdir, &["symbolic-ref", "HEAD"])).unwrap_or_else(|_| "HEAD".to_string());
    run(git_cmd(workdir, &["rev-parse", "--verify", &head_ref]))
        .map_err(|_| "You do not have the initial commit yet".to_string())?;

    commit_index_wip(workdir, &head_ref)?;
    commit_worktree_wip(workdir, &head_ref)
}

fn commit_index_wip(workdir: &Path, head_ref: &str) -> Result<(), String> {
    let wipref = format!("refs/wip/index/{head_ref}");
    let parent = wip_parent(workdir, head_ref, &wipref);
    let tree = run(git_cmd(workdir, &["write-tree"]))?;
    update_wipref(workdir, &wipref, &tree, &parent, "index")
}

fn commit_worktree_wip(workdir: &Path, head_ref: &str) -> Result<(), String> {
    let wipref = format!("refs/wip/wtree/{head_ref}");
    let parent = wip_parent(workdir, head_ref, &wipref);

    // Build the worktree tree in a temporary index seeded from the parent
    // commit, so the real index is untouched.
    let git_dir = run(git_cmd(workdir, &["rev-parse", "--absolute-git-dir"]))?;
    let temp_index = Path::new(&git_dir).join("magi-wip-index");
    let result = write_worktree_tree(workdir, &parent, &temp_index);
    let _ = std::fs::remove_file(&temp_index);
    let tree = result?;

    update_wipref(workdir, &wipref, &tree, &parent, "worktree")
}

fn write_worktree_tree(workdir: &Path, parent: &str, temp_index: &Path) -> Result<String, String> {
    let mut read_tree = git_cmd(workdir, &["read-tree", "--reset", "-i", parent]);
    read_tree.env("GIT_INDEX_FILE", temp_index);
    run(read_tree)?;

    // `add -u` records the worktree state of tracked files only, matching
    // magit-wip-commit-worktree
    let mut add = git_cmd(workdir, &["add", "-u", "."]);
    add.env("GIT_INDEX_FILE", temp_index);
    run(add)?;

    let mut write_tree = git_cmd(workdir, &["write-tree"]);
    write_tree.env("GIT_INDEX_FILE", temp_index);
    run(write_tree)
}

/// The wip ref itself is the parent when it exists and contains the current
/// branch tip; otherwise the branch tip is, and the wip ref is restarted.
fn wip_parent(workdir: &Path, head_ref: &str, wipref: &str) -> String {
    let contains_head = run(git_cmd(workdir, &["rev-parse", "--verify", wipref])).is_ok()
        && run(git_cmd(workdir, &["merge-base", wipref, head_ref]))
            == run(git_cmd(workdir, &["rev-parse", "--verify", head_ref]));
    if contains_head {
        wipref.to_string()
    } else {
        head_ref.to_string()
    }
}

fn update_wipref(
    workdir: &Path,
    wipref: &str,
    tree: &str,
    parent: &str,
    kind: &str,
) -> Result<(), String> {
    let mut parent = parent.to_string();
    if parent != wipref {
        // (Re)start the wip ref with a commit that has the branch tip's tree,
        // so every wip commit records only the wip changes as its diff
        let start_msg = format!("start autosaving {kind}");
        let parent_tree = format!("{parent}^{{tree}}");
        let commit = commit_tree(workdir, &parent_tree, &[&parent], &start_msg)?;
        update_ref(workdir, wipref, &start_msg, &commit)?;
        parent = wipref.to_string();
    }

    // Only commit when the new tree differs from the wip ref's current tree
    let unchanged = git_cmd(workdir, &["diff-tree", "--quiet", &parent, tree])
        .output()
        .map_err(|err| err.to_string())?
        .status
        .success();
    if !unchanged {
        let commit = commit_tree(workdir, tree, &[&parent], WIP_MESSAGE)?;
        update_ref(workdir, wipref, WIP_MESSAGE, &commit)?;
    }
    Ok(())
}

fn update_ref(workdir: &Path, wipref: &str, message: &str, rev: &str) -> Result<(), String> {
    run(git_cmd(
        workdir,
        &["update-ref", "--create-reflog", "-m", message, wipref, rev],
    ))
    .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;

    fn show_file(test_repo: &TestRepo, rev: &str, file: &str) -> String {
        run(git_cmd(
            test_repo.repo_path(),
            &["show", &format!("{rev}:{file}")],
        ))
        .unwrap()
    }

    /// Number of commits on the wip ref that are not on the current branch
    fn wip_commit_count(test_repo: &TestRepo, wipref: &str) -> usize {
        run(git_cmd(
            test_repo.repo_path(),
            &["rev-list", "--count", wipref, "^refs/heads/main"],
        ))
        .unwrap()
        .parse()
        .unwrap()
    }

    #[test]
    fn test_commit_to_wip_refs_records_index_and_worktree() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("file.txt", "committed", "Initial commit");
        test_repo
            .write_file_content("file.txt", "staged")
            .stage_files(&["file.txt"])
            .write_file_content("file.txt", "unstaged");

        let result = commit_to_wip_refs(test_repo.repo_path());

        assert_eq!(result, Ok(()));

        // The index wip ref records the staged state, the worktree wip ref
        // the working tree state
        assert_eq!(
            show_file(&test_repo, "refs/wip/index/refs/heads/main", "file.txt"),
            "staged"
        );
        assert_eq!(
            show_file(&test_repo, "refs/wip/wtree/refs/heads/main", "file.txt"),
            "unstaged"
        );

        // Nothing is reset: the index still holds the staged change and the
        // working tree still holds the unstaged one
        let staged = run(git_cmd(test_repo.repo_path(), &["show", ":file.txt"])).unwrap();
        assert_eq!(staged, "staged");
        let content = std::fs::read_to_string(test_repo.repo_path().join("file.txt")).unwrap();
        assert_eq!(content, "unstaged");
    }

    #[test]
    fn test_commit_to_wip_refs_appends_to_existing_refs() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("file.txt", "committed", "Initial commit");
        test_repo.write_file_content("file.txt", "first wip");
        commit_to_wip_refs(test_repo.repo_path()).unwrap();
        test_repo.write_file_content("file.txt", "second wip");

        let result = commit_to_wip_refs(test_repo.repo_path());

        assert_eq!(result, Ok(()));
        assert_eq!(
            show_file(&test_repo, "refs/wip/wtree/refs/heads/main", "file.txt"),
            "second wip"
        );
        // "start autosaving" + two wip commits
        assert_eq!(
            wip_commit_count(&test_repo, "refs/wip/wtree/refs/heads/main"),
            3
        );
    }

    #[test]
    fn test_commit_to_wip_refs_skips_commit_when_unchanged() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("file.txt", "committed", "Initial commit");
        test_repo.write_file_content("file.txt", "wip");
        commit_to_wip_refs(test_repo.repo_path()).unwrap();
        let before = wip_commit_count(&test_repo, "refs/wip/wtree/refs/heads/main");

        let result = commit_to_wip_refs(test_repo.repo_path());

        assert_eq!(result, Ok(()));
        assert_eq!(
            wip_commit_count(&test_repo, "refs/wip/wtree/refs/heads/main"),
            before
        );
    }

    #[test]
    fn test_commit_to_wip_refs_restarts_after_branch_moves() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("file.txt", "committed", "Initial commit");
        test_repo.write_file_content("file.txt", "wip");
        commit_to_wip_refs(test_repo.repo_path()).unwrap();

        // Move the branch tip past the wip ref's base, then save again
        test_repo.commit_file("file.txt", "second", "Second commit");
        test_repo.write_file_content("file.txt", "new wip");

        let result = commit_to_wip_refs(test_repo.repo_path());

        assert_eq!(result, Ok(()));
        // The wip ref was restarted on top of the new branch tip
        assert_eq!(
            show_file(&test_repo, "refs/wip/wtree/refs/heads/main", "file.txt"),
            "new wip"
        );
        assert_eq!(
            show_file(&test_repo, "refs/wip/wtree/refs/heads/main^", "file.txt"),
            "second"
        );
    }

    #[test]
    fn test_commit_to_wip_refs_without_initial_commit() {
        // TestRepo::new() always makes an initial commit, so init an empty
        // repository by hand
        let dir = tempfile::tempdir().unwrap();
        run(git_cmd(dir.path(), &["init"])).unwrap();
        std::fs::write(dir.path().join("file.txt"), "content").unwrap();

        let result = commit_to_wip_refs(dir.path());

        assert_eq!(
            result,
            Err("You do not have the initial commit yet".to_string())
        );
    }
}
