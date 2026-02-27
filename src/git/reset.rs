use git2::{Repository, StatusOptions};

/// Returns true if the repository has any uncommitted changes (staged or unstaged).
/// Untracked files are not counted.
pub fn has_uncommitted_changes(repo: &Repository) -> bool {
    let mut opts = StatusOptions::new();
    opts.include_untracked(false).include_ignored(false);
    repo.statuses(Some(&mut opts))
        .map(|statuses| !statuses.is_empty())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;

    #[test]
    fn test_has_uncommitted_changes_clean_repo() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file.txt", "content")
            .stage_files(&["file.txt"])
            .commit("Initial commit");

        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        assert!(!has_uncommitted_changes(&repo));
    }

    #[test]
    fn test_has_uncommitted_changes_with_staged_changes() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file.txt", "content")
            .stage_files(&["file.txt"])
            .commit("Initial commit");
        test_repo
            .write_file_content("file.txt", "modified")
            .stage_files(&["file.txt"]);

        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        assert!(has_uncommitted_changes(&repo));
    }

    #[test]
    fn test_has_uncommitted_changes_with_unstaged_changes() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file.txt", "content")
            .stage_files(&["file.txt"])
            .commit("Initial commit");
        test_repo.write_file_content("file.txt", "modified");

        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        assert!(has_uncommitted_changes(&repo));
    }

    #[test]
    fn test_has_uncommitted_changes_untracked_not_counted() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("file.txt", "content")
            .stage_files(&["file.txt"])
            .commit("Initial commit");
        // Write a new untracked file without staging
        test_repo.write_file_content("new_untracked.txt", "hello");

        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        assert!(!has_uncommitted_changes(&repo));
    }
}
