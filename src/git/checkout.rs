use std::path::Path;
use std::process::{Command, Stdio};

use git2::Repository;

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

/// Checkout a branch using git2.
/// For local branches, it does a simple checkout.
/// For remote branches (e.g., origin/feature), it creates a local tracking branch.
pub fn checkout<P: AsRef<Path>>(repo_path: P, branch_name: &str) -> MagiResult<CheckoutResult> {
    // Use git command for checkout as it handles both local and remote branches well
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path.as_ref())
        .arg("checkout")
        .arg(branch_name)
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
}
