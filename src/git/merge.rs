use std::path::Path;

use crate::{
    errors::MagiResult,
    git::{
        commit::{CommitResult, get_commit_result},
        git_cmd,
    },
};

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
