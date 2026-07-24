use std::collections::HashSet;
use std::path::Path;
use std::process::Stdio;

use super::git_cmd;
use crate::errors::MagiResult;

/// Result of a worktree operation
pub enum WorktreeResult {
    Success,
    Error(String),
}

/// Returns the set of local branch names currently checked out in any worktree.
/// These cannot be checked out again without `--detach`.
pub fn get_checked_out_branches<P: AsRef<Path>>(repo_path: P) -> HashSet<String> {
    let output = git_cmd(&repo_path, &["worktree", "list", "--porcelain"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    let Ok(output) = output else {
        return HashSet::new();
    };

    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .filter_map(|line| line.strip_prefix("branch refs/heads/"))
        .map(|branch| branch.to_string())
        .collect()
}

/// Returns the paths of all linked worktrees, excluding the main working
/// tree (the first entry in `git worktree list --porcelain`). The main
/// working tree cannot be moved or deleted.
pub fn list_linked_worktrees<P: AsRef<Path>>(repo_path: P) -> Vec<String> {
    let output = git_cmd(&repo_path, &["worktree", "list", "--porcelain"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    let Ok(output) = output else {
        return Vec::new();
    };

    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .filter_map(|line| line.strip_prefix("worktree "))
        .skip(1) // first entry is the main working tree
        .map(|path| path.to_string())
        .collect()
}

/// Move an existing worktree to a new location.
/// Runs: git worktree move <worktree> <new_path>
pub fn worktree_move<P: AsRef<Path>>(
    repo_path: P,
    worktree: &str,
    new_path: &str,
) -> MagiResult<WorktreeResult> {
    let output = git_cmd(&repo_path, &["worktree", "move", worktree, new_path])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        Ok(WorktreeResult::Success)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Ok(WorktreeResult::Error(if stderr.is_empty() {
            "git worktree move failed".to_string()
        } else {
            stderr
        }))
    }
}

/// Add a new worktree at `path` checking out `branch`.
/// Runs: git worktree add <path> <branch>
pub fn worktree_add<P: AsRef<Path>>(
    repo_path: P,
    path: &str,
    branch: &str,
) -> MagiResult<WorktreeResult> {
    let output = git_cmd(&repo_path, &["worktree", "add", path, branch])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        Ok(WorktreeResult::Success)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Ok(WorktreeResult::Error(if stderr.is_empty() {
            "git worktree add failed".to_string()
        } else {
            stderr
        }))
    }
}

/// Add a new worktree at `path`, creating and checking out a new branch
/// `branch` starting at `starting_point`.
/// Runs: git worktree add -b <branch> <path> <starting_point>
pub fn worktree_add_branch<P: AsRef<Path>>(
    repo_path: P,
    path: &str,
    branch: &str,
    starting_point: &str,
) -> MagiResult<WorktreeResult> {
    let output = git_cmd(
        &repo_path,
        &["worktree", "add", "-b", branch, path, starting_point],
    )
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .output()?;

    if output.status.success() {
        Ok(WorktreeResult::Success)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Ok(WorktreeResult::Error(if stderr.is_empty() {
            "git worktree add failed".to_string()
        } else {
            stderr
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;

    #[test]
    fn test_worktree_add_success() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo_path();

        // Create a branch (TestRepo::new already made an initial commit)
        let branch_name = "feature-branch";
        crate::git::git_cmd(repo_path, &["branch", branch_name])
            .output()
            .unwrap();

        // Create a unique worktree path by getting a temp path and deleting it
        // (git worktree add requires the path to not exist yet)
        let worktree_path_str = {
            let tmp = tempfile::tempdir().unwrap();
            tmp.path().to_str().unwrap().to_string()
            // tmp is dropped here, deleting the directory
        };

        let result = worktree_add(repo_path, &worktree_path_str, branch_name).unwrap();
        if let WorktreeResult::Error(ref e) = result {
            panic!("Expected success but got error: {e}");
        }
        assert!(matches!(result, WorktreeResult::Success));

        // Verify the worktree was created
        assert!(std::path::Path::new(&worktree_path_str).exists());
    }

    #[test]
    fn test_get_checked_out_branches_includes_current_branch() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo_path();

        // main is checked out after TestRepo::new()
        let checked_out = get_checked_out_branches(repo_path);
        assert!(
            checked_out.contains("main"),
            "Expected 'main' to be in checked-out branches, got: {checked_out:?}"
        );
    }

    #[test]
    fn test_get_checked_out_branches_excludes_non_checked_out() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo_path();

        crate::git::git_cmd(repo_path, &["branch", "other-branch"])
            .output()
            .unwrap();

        let checked_out = get_checked_out_branches(repo_path);
        assert!(!checked_out.contains("other-branch"));
    }

    #[test]
    fn test_worktree_add_branch_success() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo_path();

        let worktree_path_str = {
            let tmp = tempfile::tempdir().unwrap();
            tmp.path().to_str().unwrap().to_string()
        };

        let result =
            worktree_add_branch(repo_path, &worktree_path_str, "new-branch", "main").unwrap();
        if let WorktreeResult::Error(ref e) = result {
            panic!("Expected success but got error: {e}");
        }
        assert!(matches!(result, WorktreeResult::Success));

        // Verify the worktree was created and has the new branch checked out
        assert!(std::path::Path::new(&worktree_path_str).exists());
        let checked_out = get_checked_out_branches(repo_path);
        assert!(checked_out.contains("new-branch"));
    }

    #[test]
    fn test_worktree_add_branch_existing_branch_returns_error() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo_path();

        let worktree_path_str = {
            let tmp = tempfile::tempdir().unwrap();
            tmp.path().to_str().unwrap().to_string()
        };

        // -b refuses to overwrite an existing branch
        let result = worktree_add_branch(repo_path, &worktree_path_str, "main", "main").unwrap();
        assert!(matches!(result, WorktreeResult::Error(_)));
    }

    #[test]
    fn test_worktree_add_branch_invalid_starting_point_returns_error() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo_path();

        let worktree_path_str = {
            let tmp = tempfile::tempdir().unwrap();
            tmp.path().to_str().unwrap().to_string()
        };

        let result = worktree_add_branch(
            repo_path,
            &worktree_path_str,
            "new-branch",
            "nonexistent-ref",
        )
        .unwrap();
        assert!(matches!(result, WorktreeResult::Error(_)));
    }

    #[test]
    fn test_list_linked_worktrees_excludes_main() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo_path();

        // Only the main working tree exists
        assert!(list_linked_worktrees(repo_path).is_empty());

        crate::git::git_cmd(repo_path, &["branch", "feature"])
            .output()
            .unwrap();
        let worktree_path_str = {
            let tmp = tempfile::tempdir().unwrap();
            tmp.path().to_str().unwrap().to_string()
        };
        worktree_add(repo_path, &worktree_path_str, "feature").unwrap();

        let worktrees = list_linked_worktrees(repo_path);
        assert_eq!(worktrees.len(), 1);
        assert_eq!(
            std::path::Path::new(&worktrees[0]).canonicalize().unwrap(),
            std::path::Path::new(&worktree_path_str)
                .canonicalize()
                .unwrap()
        );
    }

    #[test]
    fn test_worktree_move_success() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo_path();

        crate::git::git_cmd(repo_path, &["branch", "feature"])
            .output()
            .unwrap();
        let old_path = {
            let tmp = tempfile::tempdir().unwrap();
            tmp.path().to_str().unwrap().to_string()
        };
        worktree_add(repo_path, &old_path, "feature").unwrap();

        let new_path = {
            let tmp = tempfile::tempdir().unwrap();
            tmp.path().to_str().unwrap().to_string()
        };

        let result = worktree_move(repo_path, &old_path, &new_path).unwrap();
        if let WorktreeResult::Error(ref e) = result {
            panic!("Expected success but got error: {e}");
        }
        assert!(!std::path::Path::new(&old_path).exists());
        assert!(std::path::Path::new(&new_path).exists());

        let worktrees = list_linked_worktrees(repo_path);
        assert_eq!(worktrees.len(), 1);
        assert_eq!(
            std::path::Path::new(&worktrees[0]).canonicalize().unwrap(),
            std::path::Path::new(&new_path).canonicalize().unwrap()
        );
    }

    #[test]
    fn test_worktree_move_main_worktree_returns_error() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo_path();

        let new_path = {
            let tmp = tempfile::tempdir().unwrap();
            tmp.path().to_str().unwrap().to_string()
        };

        let result = worktree_move(repo_path, repo_path.to_str().unwrap(), &new_path).unwrap();
        assert!(matches!(result, WorktreeResult::Error(_)));
    }

    #[test]
    fn test_worktree_move_nonexistent_worktree_returns_error() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo_path();

        let result = worktree_move(repo_path, "/nonexistent/worktree", "/tmp/nowhere").unwrap();
        assert!(matches!(result, WorktreeResult::Error(_)));
    }

    #[test]
    fn test_worktree_add_invalid_branch_returns_error() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo_path();

        let worktree_path_str = {
            let tmp = tempfile::tempdir().unwrap();
            tmp.path().to_str().unwrap().to_string()
        };

        let result = worktree_add(repo_path, &worktree_path_str, "nonexistent-branch").unwrap();
        assert!(matches!(result, WorktreeResult::Error(_)));
    }
}
