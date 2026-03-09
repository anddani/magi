use std::fs;
use std::path::Path;

/// Returns true if a cherry-pick sequence is currently in progress.
/// Checks for CHERRY_PICK_HEAD (stopped on conflict) or sequencer/todo starting with "pick".
pub fn cherry_pick_in_progress(workdir: &Path) -> bool {
    let git_dir = workdir.join(".git");

    // A stopped cherry-pick creates CHERRY_PICK_HEAD
    if git_dir.join("CHERRY_PICK_HEAD").exists() {
        return true;
    }

    // A multi-commit cherry-pick sequence writes sequencer/todo
    let todo_path = git_dir.join("sequencer").join("todo");
    if todo_path.exists()
        && let Ok(content) = fs::read_to_string(&todo_path)
        && let Some(first_line) = content.lines().next()
        && first_line.trim_start().starts_with("pick")
    {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cherry_pick_in_progress_no_files() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!cherry_pick_in_progress(dir.path()));
    }

    #[test]
    fn test_cherry_pick_in_progress_with_cherry_pick_head() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("CHERRY_PICK_HEAD"), "abc1234\n").unwrap();
        assert!(cherry_pick_in_progress(dir.path()));
    }

    #[test]
    fn test_cherry_pick_in_progress_with_sequencer_todo() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let sequencer = git_dir.join("sequencer");
        fs::create_dir_all(&sequencer).unwrap();
        fs::write(
            sequencer.join("todo"),
            "pick abc1234 # Some commit message\n",
        )
        .unwrap();
        assert!(cherry_pick_in_progress(dir.path()));
    }

    #[test]
    fn test_cherry_pick_in_progress_sequencer_not_pick() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let sequencer = git_dir.join("sequencer");
        fs::create_dir_all(&sequencer).unwrap();
        fs::write(
            sequencer.join("todo"),
            "revert abc1234 # Some commit message\n",
        )
        .unwrap();
        assert!(!cherry_pick_in_progress(dir.path()));
    }
}
