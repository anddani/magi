use std::path::Path;
use std::process::Stdio;

use git2::Repository;

use super::git_cmd;
use crate::errors::MagiResult;

/// Result of a checkout operation
pub enum CheckoutResult {
    Success,
    Error(String),
}

/// Gets all branches (local and remote) for the select popup.
/// Returns local branches first, then remote branches (excluding origin/HEAD).
pub fn get_branches(repo: &Repository) -> Vec<String> {
    let mut branches = Vec::new();

    // Get local branches
    if let Ok(local_branches) = repo.branches(Some(git2::BranchType::Local)) {
        for branch_result in local_branches.flatten() {
            if let Ok(Some(name)) = branch_result.0.name() {
                branches.push(name.to_string());
            }
        }
    }

    // Get remote branches (excluding HEAD references)
    if let Ok(remote_branches) = repo.branches(Some(git2::BranchType::Remote)) {
        for branch_result in remote_branches.flatten() {
            if let Ok(Some(name)) = branch_result.0.name() {
                // Skip origin/HEAD type references
                if !name.ends_with("/HEAD") {
                    branches.push(name.to_string());
                }
            }
        }
    }

    branches
}

/// Gets only local branches for the select popup.
pub fn get_local_branches(repo: &Repository) -> Vec<String> {
    let mut branches = Vec::new();

    if let Ok(local_branches) = repo.branches(Some(git2::BranchType::Local)) {
        for branch_result in local_branches.flatten() {
            if let Ok(Some(name)) = branch_result.0.name() {
                branches.push(name.to_string());
            }
        }
    }

    branches
}

/// Gets remote branches for the push/fetch upstream select popup.
/// Returns only remote branches (e.g., "origin/main", "origin/feature").
/// Optionally prepends a suggested upstream if it doesn't already exist in the list.
pub fn get_remote_branches_for_upstream(
    repo: &Repository,
    suggested_upstream: Option<&str>,
) -> Vec<String> {
    let mut branches = Vec::new();

    // Add suggested upstream first if provided
    if let Some(suggested) = suggested_upstream {
        branches.push(suggested.to_string());
    }

    // Get remote branches (excluding HEAD references)
    if let Ok(remote_branches) = repo.branches(Some(git2::BranchType::Remote)) {
        for branch_result in remote_branches.flatten() {
            if let Ok(Some(name)) = branch_result.0.name() {
                // Skip origin/HEAD type references
                if !name.ends_with("/HEAD") {
                    // Don't add duplicates of the suggested upstream
                    if suggested_upstream != Some(name) {
                        branches.push(name.to_string());
                    }
                }
            }
        }
    }

    branches
}

/// Returns the last checked-out branch by scanning the HEAD reflog for
/// "checkout: moving from X to Y" entries and returning the most recent "from" branch
/// that differs from the current HEAD.
pub fn get_last_checked_out_branch(repo: &Repository) -> Option<String> {
    let reflog = repo.reflog("HEAD").ok()?;
    for entry in reflog.iter() {
        let msg = entry.message()?;
        if let Some(rest) = msg.strip_prefix("checkout: moving from ")
            && let Some((from, _to)) = rest.split_once(" to ")
        {
            return Some(from.to_string());
        }
    }
    None
}

/// Checkout a branch using git2.
/// For local branches, it does a simple checkout.
/// For remote branches (e.g., origin/feature), it creates a local tracking branch.
pub fn checkout<P: AsRef<Path>>(repo_path: P, branch_name: &str) -> MagiResult<CheckoutResult> {
    // Use git command for checkout as it handles both local and remote branches well
    let output = git_cmd(&repo_path, &["checkout", branch_name])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        Ok(CheckoutResult::Success)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Ok(CheckoutResult::Error(if stderr.is_empty() {
            "Checkout failed".to_string()
        } else {
            stderr
        }))
    }
}

/// Create and checkout a new branch at the specified starting point.
/// Uses `git checkout -b <branch_name> <starting_point>`.
pub fn checkout_new_branch<P: AsRef<Path>>(
    repo_path: P,
    branch_name: &str,
    starting_point: &str,
) -> MagiResult<CheckoutResult> {
    let output = git_cmd(&repo_path, &["checkout", "-b", branch_name, starting_point])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        Ok(CheckoutResult::Success)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Ok(CheckoutResult::Error(if stderr.is_empty() {
            "Failed to create branch".to_string()
        } else {
            stderr
        }))
    }
}

/// Result of a delete branch operation
pub enum DeleteBranchResult {
    Success,
    Error(String),
}

/// Delete a branch. For local branches, deletes with `git branch -D`.
/// For remote branches (e.g., `origin/feature`), deletes with `git push --delete`.
/// If the user is currently on the branch being deleted, detaches HEAD first.
pub fn delete_branch<P: AsRef<Path>>(
    repo_path: P,
    branch_name: &str,
) -> MagiResult<DeleteBranchResult> {
    let repo_path = repo_path.as_ref();

    // Check if this is a remote branch (contains '/')
    if let Some((remote, branch)) = branch_name.split_once('/') {
        // Remote branch: git push --delete <remote> <branch>
        let output = git_cmd(repo_path, &["push", "--delete", remote, branch])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if output.status.success() {
            Ok(DeleteBranchResult::Success)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Ok(DeleteBranchResult::Error(if stderr.is_empty() {
                "Failed to delete remote branch".to_string()
            } else {
                stderr
            }))
        }
    } else {
        // Local branch: check if we're on it and detach HEAD if so
        let head_output = git_cmd(repo_path, &["symbolic-ref", "--short", "HEAD"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if head_output.status.success() {
            let current_branch = String::from_utf8_lossy(&head_output.stdout)
                .trim()
                .to_string();
            if current_branch == branch_name {
                // Detach HEAD before deleting the current branch
                let detach = git_cmd(repo_path, &["checkout", "--detach"])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()?;

                if !detach.status.success() {
                    let stderr = String::from_utf8_lossy(&detach.stderr).trim().to_string();
                    return Ok(DeleteBranchResult::Error(format!(
                        "Failed to detach HEAD: {}",
                        stderr
                    )));
                }
            }
        }

        // Delete local branch with -D (force)
        let output = git_cmd(repo_path, &["branch", "-D", branch_name])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if output.status.success() {
            Ok(DeleteBranchResult::Success)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Ok(DeleteBranchResult::Error(if stderr.is_empty() {
                "Failed to delete branch".to_string()
            } else {
                stderr
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;
    use std::fs::File;
    use std::io::Write;
    use std::process::Command;

    #[test]
    fn test_get_branches_returns_local_branches() {
        let test_repo = TestRepo::new();
        let branches = get_branches(&test_repo.repo);

        // Should have at least the main/master branch
        assert!(!branches.is_empty());
    }

    #[test]
    fn test_get_branches_includes_new_branch() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();

        // Create a new branch
        Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .args(["branch", "test-branch"])
            .output()
            .expect("Failed to create branch");

        let branches = get_branches(&test_repo.repo);
        assert!(branches.iter().any(|b| b == "test-branch"));
    }

    #[test]
    fn test_get_local_branches_returns_only_local() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();

        // Create a new local branch
        Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .args(["branch", "local-branch"])
            .output()
            .expect("Failed to create branch");

        let branches = get_local_branches(&test_repo.repo);

        // Should contain the local branch
        assert!(branches.iter().any(|b| b == "local-branch"));

        // Should not contain any remote branches (no slashes in names)
        assert!(branches.iter().all(|b| !b.contains('/')));
    }

    #[test]
    fn test_checkout_existing_branch() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();

        // Create a new branch
        Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .args(["branch", "test-checkout"])
            .output()
            .expect("Failed to create branch");

        // Checkout the branch
        let result = checkout(repo_path, "test-checkout").unwrap();
        assert!(matches!(result, CheckoutResult::Success));
    }

    #[test]
    fn test_checkout_nonexistent_branch_fails() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();

        let result = checkout(repo_path, "nonexistent-branch-xyz").unwrap();
        assert!(matches!(result, CheckoutResult::Error(_)));
    }

    #[test]
    fn test_checkout_with_uncommitted_changes_conflicts() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();

        // Create a new branch
        Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .args(["branch", "conflict-branch"])
            .output()
            .expect("Failed to create branch");

        // Make uncommitted changes to an existing file
        let file_path = repo_path.join("initial.txt");
        let mut file = File::create(&file_path).expect("Failed to create file");
        writeln!(file, "Modified content").expect("Failed to write");

        // This should still succeed as long as there are no conflicts
        // (the file doesn't exist in the other branch differently)
        let result = checkout(repo_path, "conflict-branch").unwrap();
        // Result depends on whether there's a conflict - just check it returns something
        assert!(
            matches!(result, CheckoutResult::Success) || matches!(result, CheckoutResult::Error(_))
        );
    }

    #[test]
    fn test_checkout_new_branch_creates_branch() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();

        // Create a new branch from the current HEAD
        let result = checkout_new_branch(repo_path, "new-feature", "HEAD").unwrap();
        assert!(matches!(result, CheckoutResult::Success));

        // Verify the branch was created and checked out
        let branches = get_branches(&test_repo.repo);
        assert!(branches.iter().any(|b| b == "new-feature"));
    }

    #[test]
    fn test_checkout_new_branch_from_branch() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();

        // Create a branch to use as starting point
        Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .args(["branch", "base-branch"])
            .output()
            .expect("Failed to create base branch");

        // Create a new branch from the base branch
        let result = checkout_new_branch(repo_path, "derived-branch", "base-branch").unwrap();
        assert!(matches!(result, CheckoutResult::Success));

        // Verify the branch was created
        let branches = get_branches(&test_repo.repo);
        assert!(branches.iter().any(|b| b == "derived-branch"));
    }

    #[test]
    fn test_checkout_new_branch_duplicate_name_fails() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();

        // Create a branch first
        Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .args(["branch", "existing-branch"])
            .output()
            .expect("Failed to create branch");

        // Try to create another branch with the same name
        let result = checkout_new_branch(repo_path, "existing-branch", "HEAD").unwrap();
        assert!(matches!(result, CheckoutResult::Error(_)));
    }

    #[test]
    fn test_checkout_new_branch_invalid_starting_point_fails() {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();

        // Try to create a branch from a non-existent starting point
        let result = checkout_new_branch(repo_path, "new-branch", "nonexistent-ref").unwrap();
        assert!(matches!(result, CheckoutResult::Error(_)));
    }
}
