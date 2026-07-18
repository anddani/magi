use std::path::Path;

use crate::{
    errors::MagiResult,
    git::{
        commit::{CommitResult, get_commit_result},
        git_cmd,
    },
};

/// Lists paths with unresolved conflicts
/// (`git diff --name-only --diff-filter=U`).
pub fn conflicted_files<P: AsRef<Path>>(repo_path: P) -> MagiResult<Vec<String>> {
    let output = git_cmd(&repo_path, &["diff", "--name-only", "--diff-filter=U"]).output()?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

pub fn run_merge_continue_with_editor<P: AsRef<Path>>(repo_path: P) -> MagiResult<CommitResult> {
    let status = git_cmd(&repo_path, &["merge", "--continue"]).status()?;

    get_commit_result(repo_path, status, "Merge continue")
}

/// Runs `git merge <branch>`, which may open the user's configured editor
/// for the merge commit message. The caller must ensure the TUI is suspended
/// (via `RunningState::LaunchExternalCommand`) before calling this.
pub fn run_merge_with_editor<P: AsRef<Path>>(
    repo_path: P,
    branch: &str,
) -> MagiResult<CommitResult> {
    let status = git_cmd(&repo_path, &["merge", branch]).status()?;

    get_commit_result(repo_path, status, "Merge")
}

/// Runs `git merge --edit --no-ff <branch>`, which always creates a merge
/// commit and opens the user's configured editor for its message. The caller
/// must ensure the TUI is suspended (via `RunningState::LaunchExternalCommand`)
/// before calling this.
pub fn run_merge_edit_with_editor<P: AsRef<Path>>(
    repo_path: P,
    branch: &str,
) -> MagiResult<CommitResult> {
    let status = git_cmd(&repo_path, &["merge", "--edit", "--no-ff", branch]).status()?;

    get_commit_result(repo_path, status, "Merge")
}

/// Runs `git merge --no-edit <branch>` and, when the merge succeeds, deletes
/// the merged branch (`git branch -D <branch>`). `--no-edit` never opens an
/// editor, so the TUI does not need to be suspended. When the merge stops on
/// conflicts the branch is kept so the merge can be resolved or aborted.
pub fn run_merge_absorb<P: AsRef<Path>>(repo_path: P, branch: &str) -> MagiResult<CommitResult> {
    let repo_path = repo_path.as_ref();
    let output = git_cmd(repo_path, &["merge", "--no-edit", branch]).output()?;

    let result = get_commit_result(repo_path, output.status, "Absorb")?;
    if !result.success {
        return Ok(result);
    }

    let delete = git_cmd(repo_path, &["branch", "-D", branch]).output()?;
    if delete.status.success() {
        Ok(result)
    } else {
        let stderr = String::from_utf8_lossy(&delete.stderr).trim().to_string();
        Ok(CommitResult {
            success: false,
            message: format!("Absorbed '{}' but failed to delete it: {}", branch, stderr),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::{git_cmd, test_repo::TestRepo};
    use std::fs;

    fn run_git(test_repo: &TestRepo, args: &[&str]) -> std::process::Output {
        git_cmd(test_repo.repo_path(), args).output().unwrap()
    }

    /// `git merge --continue` opens an editor for the merge commit message.
    /// Point core.editor at `true` so the test does not hang waiting for input.
    fn disable_editor(test_repo: &TestRepo) {
        test_repo
            .repo
            .config()
            .unwrap()
            .set_str("core.editor", "true")
            .unwrap();
    }

    fn merge_in_progress(test_repo: &TestRepo) -> bool {
        test_repo.repo.path().join("MERGE_HEAD").exists()
    }

    /// Creates `feature` diverging from `main`: both branches get one commit
    /// after the shared base. Leaves the repo checked out on `main`.
    fn setup_divergent_branches(
        test_repo: &TestRepo,
        main_file: (&str, &str),
        feature_file: (&str, &str),
    ) {
        test_repo.commit_file("base.txt", "base\n", "Base commit");
        test_repo.create_branch("feature");
        test_repo.commit_file(main_file.0, main_file.1, "Main commit");
        assert!(
            run_git(test_repo, &["checkout", "feature"])
                .status
                .success()
        );
        test_repo.commit_file(feature_file.0, feature_file.1, "Feature commit");
        assert!(run_git(test_repo, &["checkout", "main"]).status.success());
    }

    #[test]
    fn test_conflicted_files_empty_when_no_conflicts() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("base.txt", "base\n", "Base commit");

        assert!(conflicted_files(test_repo.repo_path()).unwrap().is_empty());
    }

    #[test]
    fn test_conflicted_files_lists_unresolved_paths() {
        let test_repo = TestRepo::new();
        disable_editor(&test_repo);
        // Both branches modify the same file: merging conflicts.
        setup_divergent_branches(
            &test_repo,
            ("base.txt", "main change\n"),
            ("base.txt", "feature change\n"),
        );

        let merge = run_git(&test_repo, &["merge", "feature"]);
        assert!(!merge.status.success());

        assert_eq!(
            conflicted_files(test_repo.repo_path()).unwrap(),
            vec!["base.txt".to_string()]
        );

        // Resolving and staging the file clears the conflict list.
        test_repo
            .write_file_content("base.txt", "resolved\n")
            .stage_files(&["base.txt"]);
        assert!(conflicted_files(test_repo.repo_path()).unwrap().is_empty());
    }

    #[test]
    fn test_merge_branch_fast_forward_succeeds() {
        let test_repo = TestRepo::new();
        disable_editor(&test_repo);
        test_repo.commit_file("base.txt", "base\n", "Base commit");

        // Put a commit on feature only, so merging it into main fast-forwards.
        assert!(
            run_git(&test_repo, &["checkout", "-b", "feature"])
                .status
                .success()
        );
        test_repo.commit_file("feature.txt", "feature content\n", "Feature commit");
        assert!(run_git(&test_repo, &["checkout", "main"]).status.success());

        let result = run_merge_with_editor(test_repo.repo_path(), "feature").unwrap();

        assert!(result.success);
        assert_eq!(result.message, "Merge: Feature commit");
        assert_eq!(test_repo.head_hash(), test_repo.branch_hash("feature"));
        assert!(!merge_in_progress(&test_repo));
    }

    #[test]
    fn test_merge_branch_divergent_creates_merge_commit() {
        let test_repo = TestRepo::new();
        disable_editor(&test_repo);
        // Divergent branches touching different files: merges cleanly.
        setup_divergent_branches(
            &test_repo,
            ("main.txt", "main content\n"),
            ("feature.txt", "feature content\n"),
        );

        let result = run_merge_with_editor(test_repo.repo_path(), "feature").unwrap();

        assert!(result.success);
        assert!(
            result.message.starts_with("Merge: Merge branch 'feature'"),
            "unexpected message: {}",
            result.message
        );

        let head = test_repo.repo.head().unwrap().peel_to_commit().unwrap();
        assert_eq!(head.parent_count(), 2);
        assert!(test_repo.repo_path().join("main.txt").exists());
        assert!(test_repo.repo_path().join("feature.txt").exists());
        assert!(!merge_in_progress(&test_repo));
    }

    #[test]
    fn test_merge_edit_fast_forward_still_creates_merge_commit() {
        let test_repo = TestRepo::new();
        disable_editor(&test_repo);
        test_repo.commit_file("base.txt", "base\n", "Base commit");

        // Put a commit on feature only, so a plain merge would fast-forward.
        assert!(
            run_git(&test_repo, &["checkout", "-b", "feature"])
                .status
                .success()
        );
        test_repo.commit_file("feature.txt", "feature content\n", "Feature commit");
        assert!(run_git(&test_repo, &["checkout", "main"]).status.success());

        let result = run_merge_edit_with_editor(test_repo.repo_path(), "feature").unwrap();

        assert!(result.success);
        assert!(
            result.message.starts_with("Merge: Merge branch 'feature'"),
            "unexpected message: {}",
            result.message
        );

        // --no-ff forces a merge commit even when fast-forward is possible.
        let head = test_repo.repo.head().unwrap().peel_to_commit().unwrap();
        assert_eq!(head.parent_count(), 2);
        assert!(test_repo.repo_path().join("feature.txt").exists());
        assert!(!merge_in_progress(&test_repo));
    }

    #[test]
    fn test_merge_edit_divergent_creates_merge_commit() {
        let test_repo = TestRepo::new();
        disable_editor(&test_repo);
        setup_divergent_branches(
            &test_repo,
            ("main.txt", "main content\n"),
            ("feature.txt", "feature content\n"),
        );

        let result = run_merge_edit_with_editor(test_repo.repo_path(), "feature").unwrap();

        assert!(result.success);
        assert!(
            result.message.starts_with("Merge: Merge branch 'feature'"),
            "unexpected message: {}",
            result.message
        );

        let head = test_repo.repo.head().unwrap().peel_to_commit().unwrap();
        assert_eq!(head.parent_count(), 2);
        assert!(test_repo.repo_path().join("main.txt").exists());
        assert!(test_repo.repo_path().join("feature.txt").exists());
        assert!(!merge_in_progress(&test_repo));
    }

    #[test]
    fn test_merge_edit_with_conflicts_fails() {
        let test_repo = TestRepo::new();
        disable_editor(&test_repo);
        setup_divergent_branches(
            &test_repo,
            ("base.txt", "main change\n"),
            ("base.txt", "feature change\n"),
        );

        let result = run_merge_edit_with_editor(test_repo.repo_path(), "feature").unwrap();

        assert!(!result.success);
        assert_eq!(result.message, "Merge aborted");
        assert!(merge_in_progress(&test_repo));
    }

    #[test]
    fn test_merge_branch_with_conflicts_fails() {
        let test_repo = TestRepo::new();
        disable_editor(&test_repo);
        // Both branches modify the same file: merging conflicts.
        setup_divergent_branches(
            &test_repo,
            ("base.txt", "main change\n"),
            ("base.txt", "feature change\n"),
        );

        let result = run_merge_with_editor(test_repo.repo_path(), "feature").unwrap();

        assert!(!result.success);
        assert_eq!(result.message, "Merge aborted");
        assert!(merge_in_progress(&test_repo));
    }

    #[test]
    fn test_merge_absorb_merges_and_deletes_branch() {
        let test_repo = TestRepo::new();
        // Divergent branches touching different files: merges cleanly.
        setup_divergent_branches(
            &test_repo,
            ("main.txt", "main content\n"),
            ("feature.txt", "feature content\n"),
        );

        let result = run_merge_absorb(test_repo.repo_path(), "feature").unwrap();

        assert!(result.success);
        assert!(
            result.message.starts_with("Absorb: Merge branch 'feature'"),
            "unexpected message: {}",
            result.message
        );

        let head = test_repo.repo.head().unwrap().peel_to_commit().unwrap();
        assert_eq!(head.parent_count(), 2);
        assert!(test_repo.repo_path().join("main.txt").exists());
        assert!(test_repo.repo_path().join("feature.txt").exists());
        assert!(!merge_in_progress(&test_repo));
        // The absorbed branch is gone.
        assert!(
            test_repo
                .repo
                .find_branch("feature", git2::BranchType::Local)
                .is_err()
        );
    }

    #[test]
    fn test_merge_absorb_fast_forward_deletes_branch() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("base.txt", "base\n", "Base commit");

        // Put a commit on feature only, so merging it into main fast-forwards.
        assert!(
            run_git(&test_repo, &["checkout", "-b", "feature"])
                .status
                .success()
        );
        test_repo.commit_file("feature.txt", "feature content\n", "Feature commit");
        assert!(run_git(&test_repo, &["checkout", "main"]).status.success());
        let feature_hash = test_repo.branch_hash("feature");

        let result = run_merge_absorb(test_repo.repo_path(), "feature").unwrap();

        assert!(result.success);
        assert_eq!(result.message, "Absorb: Feature commit");
        assert_eq!(test_repo.head_hash(), feature_hash);
        assert!(!merge_in_progress(&test_repo));
        assert!(
            test_repo
                .repo
                .find_branch("feature", git2::BranchType::Local)
                .is_err()
        );
    }

    #[test]
    fn test_merge_absorb_with_conflicts_keeps_branch() {
        let test_repo = TestRepo::new();
        // Both branches modify the same file: merging conflicts.
        setup_divergent_branches(
            &test_repo,
            ("base.txt", "main change\n"),
            ("base.txt", "feature change\n"),
        );

        let result = run_merge_absorb(test_repo.repo_path(), "feature").unwrap();

        assert!(!result.success);
        assert_eq!(result.message, "Absorb aborted");
        assert!(merge_in_progress(&test_repo));
        // The branch survives so the merge can be resolved or aborted.
        assert!(
            test_repo
                .repo
                .find_branch("feature", git2::BranchType::Local)
                .is_ok()
        );
    }

    #[test]
    fn test_merge_continue_without_merge_in_progress_fails() {
        let test_repo = TestRepo::new();
        disable_editor(&test_repo);
        test_repo.commit_file("file.txt", "content", "First commit");

        let result = run_merge_continue_with_editor(test_repo.repo_path()).unwrap();

        assert!(!result.success);
        assert_eq!(result.message, "Merge continue aborted");
    }

    #[test]
    fn test_merge_continue_completes_merge_of_divergent_branches() {
        let test_repo = TestRepo::new();
        disable_editor(&test_repo);
        // Divergent branches touching different files: merges cleanly.
        setup_divergent_branches(
            &test_repo,
            ("main.txt", "main content\n"),
            ("feature.txt", "feature content\n"),
        );

        // Start a merge but stop before committing, leaving MERGE_HEAD in place.
        let merge = run_git(&test_repo, &["merge", "--no-commit", "--no-ff", "feature"]);
        assert!(merge.status.success());
        assert!(merge_in_progress(&test_repo));

        let result = run_merge_continue_with_editor(test_repo.repo_path()).unwrap();

        assert!(result.success);
        assert!(
            result
                .message
                .starts_with("Merge continue: Merge branch 'feature'"),
            "unexpected message: {}",
            result.message
        );

        // The merge commit has two parents and contains both branches' files.
        let head = test_repo.repo.head().unwrap().peel_to_commit().unwrap();
        assert_eq!(head.parent_count(), 2);
        assert!(test_repo.repo_path().join("main.txt").exists());
        assert!(test_repo.repo_path().join("feature.txt").exists());
        assert!(!merge_in_progress(&test_repo));
    }

    #[test]
    fn test_merge_continue_with_unresolved_conflicts_fails() {
        let test_repo = TestRepo::new();
        disable_editor(&test_repo);
        // Both branches modify the same file: merging conflicts.
        setup_divergent_branches(
            &test_repo,
            ("base.txt", "main change\n"),
            ("base.txt", "feature change\n"),
        );

        let merge = run_git(&test_repo, &["merge", "feature"]);
        assert!(!merge.status.success());
        assert!(merge_in_progress(&test_repo));

        // Continuing without resolving the conflict must fail and keep the
        // merge in progress.
        let result = run_merge_continue_with_editor(test_repo.repo_path()).unwrap();

        assert!(!result.success);
        assert_eq!(result.message, "Merge continue aborted");
        assert!(merge_in_progress(&test_repo));
    }

    #[test]
    fn test_merge_continue_after_resolving_conflicts_succeeds() {
        let test_repo = TestRepo::new();
        disable_editor(&test_repo);
        setup_divergent_branches(
            &test_repo,
            ("base.txt", "main change\n"),
            ("base.txt", "feature change\n"),
        );

        let merge = run_git(&test_repo, &["merge", "feature"]);
        assert!(!merge.status.success());

        // Resolve the conflict and stage the resolution.
        test_repo
            .write_file_content("base.txt", "resolved\n")
            .stage_files(&["base.txt"]);

        let result = run_merge_continue_with_editor(test_repo.repo_path()).unwrap();

        assert!(result.success);
        assert!(
            result
                .message
                .starts_with("Merge continue: Merge branch 'feature'"),
            "unexpected message: {}",
            result.message
        );

        let head = test_repo.repo.head().unwrap().peel_to_commit().unwrap();
        assert_eq!(head.parent_count(), 2);
        assert_eq!(
            fs::read_to_string(test_repo.repo_path().join("base.txt")).unwrap(),
            "resolved\n"
        );
        assert!(!merge_in_progress(&test_repo));
    }

    #[test]
    fn test_merge_continue_after_fast_forward_merge_fails() {
        let test_repo = TestRepo::new();
        disable_editor(&test_repo);
        test_repo.commit_file("base.txt", "base\n", "Base commit");

        // Put a commit on feature only, so merging it into main fast-forwards.
        assert!(
            run_git(&test_repo, &["checkout", "-b", "feature"])
                .status
                .success()
        );
        test_repo.commit_file("feature.txt", "feature content\n", "Feature commit");
        assert!(run_git(&test_repo, &["checkout", "main"]).status.success());

        let merge = run_git(&test_repo, &["merge", "feature"]);
        assert!(merge.status.success());
        assert_eq!(test_repo.head_hash(), test_repo.branch_hash("feature"));

        // A fast-forward merge completes immediately, so there is no merge in
        // progress left to continue.
        assert!(!merge_in_progress(&test_repo));
        let result = run_merge_continue_with_editor(test_repo.repo_path()).unwrap();
        assert!(!result.success);
        assert_eq!(result.message, "Merge continue aborted");
    }
}
