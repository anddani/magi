use std::path::Path;
use std::process::Stdio;

use super::git_cmd;
use crate::errors::MagiResult;

/// Result of a worktree add operation
pub enum WorktreeAddResult {
    Success,
    Error(String),
}

/// Add a new worktree at `path` checking out `branch`.
/// Runs: git worktree add <path> <branch>
pub fn worktree_add<P: AsRef<Path>>(
    repo_path: P,
    path: &str,
    branch: &str,
) -> MagiResult<WorktreeAddResult> {
    let output = git_cmd(&repo_path, &["worktree", "add", path, branch])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        Ok(WorktreeAddResult::Success)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Ok(WorktreeAddResult::Error(if stderr.is_empty() {
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
        if let WorktreeAddResult::Error(ref e) = result {
            panic!("Expected success but got error: {e}");
        }
        assert!(matches!(result, WorktreeAddResult::Success));

        // Verify the worktree was created
        assert!(std::path::Path::new(&worktree_path_str).exists());
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
        assert!(matches!(result, WorktreeAddResult::Error(_)));
    }
}
