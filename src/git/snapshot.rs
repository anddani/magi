use std::path::Path;

use crate::git::{git_cmd, read_commit_message};

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
}
