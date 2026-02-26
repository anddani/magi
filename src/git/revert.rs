use std::fs;
use std::path::Path;
use std::process::Command;

use crate::{
    errors::MagiResult,
    model::{LineContent, SectionType},
};

/// Returns true if a revert sequence is currently in progress.
/// Checks for REVERT_HEAD or a sequencer/todo file starting with "revert".
pub fn revert_in_progress(workdir: &Path) -> bool {
    let git_dir = workdir.join(".git");

    // A stopped revert creates REVERT_HEAD
    if git_dir.join("REVERT_HEAD").exists() {
        return true;
    }

    // A multi-commit revert sequence writes sequencer/todo
    let todo_path = git_dir.join("sequencer").join("todo");
    if todo_path.exists()
        && let Ok(content) = fs::read_to_string(&todo_path)
        && let Some(first_line) = content.lines().next()
        && first_line.trim_start().starts_with("revert")
    {
        return true;
    }

    false
}

/// A single entry shown in the "Reverting" section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevertingEntry {
    pub hash: String,
    pub message: String,
    /// true = the commit currently stopped on (REVERT_HEAD); false = pending in sequencer
    pub is_current: bool,
}

/// Returns the list of reverting entries when a revert sequence is in progress.
/// The current commit (REVERT_HEAD) comes first, followed by pending sequencer entries.
pub fn get_reverting_entries(workdir: &Path) -> Vec<RevertingEntry> {
    let git_dir = workdir.join(".git");
    let mut entries = Vec::new();

    // Current stopped commit from REVERT_HEAD
    let revert_head_path = git_dir.join("REVERT_HEAD");
    if let Ok(hash_raw) = fs::read_to_string(&revert_head_path) {
        let hash = hash_raw.trim().to_string();
        if !hash.is_empty() {
            let short_hash: String = hash.chars().take(7).collect();
            let message = read_commit_message(workdir, &hash).unwrap_or_default();
            entries.push(RevertingEntry {
                hash: short_hash,
                message,
                is_current: true,
            });
        }
    }

    // Pending commits from sequencer/todo
    let todo_path = git_dir.join("sequencer").join("todo");
    if let Ok(content) = fs::read_to_string(&todo_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Format: "revert <full-hash> # <message>"
            if let Some(rest) = line.strip_prefix("revert ") {
                let parts: Vec<&str> = rest.splitn(2, " # ").collect();
                let short_hash: String = parts[0].trim().chars().take(7).collect();
                let message = parts
                    .get(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                entries.push(RevertingEntry {
                    hash: short_hash,
                    message,
                    is_current: false,
                });
            }
        }
    }

    entries
}

/// Reads the subject line of a commit from git log.
fn read_commit_message(workdir: &Path, hash: &str) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workdir)
        .args(["log", "--format=%s", "-1", hash])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Returns model lines for the "Reverting" section.
/// Returns an empty vec if no revert is in progress.
pub fn get_reverting_lines(workdir: &Path) -> MagiResult<Vec<crate::model::Line>> {
    if !revert_in_progress(workdir) {
        return Ok(vec![]);
    }

    let entries = get_reverting_entries(workdir);
    if entries.is_empty() {
        return Ok(vec![]);
    }

    let mut lines = Vec::new();

    // Section header
    lines.push(crate::model::Line {
        content: LineContent::SectionHeader {
            title: "Reverting".to_string(),
            count: None,
        },
        section: Some(SectionType::Reverting),
    });

    for entry in entries {
        lines.push(crate::model::Line {
            content: LineContent::RevertingEntry {
                hash: entry.hash,
                message: entry.message,
                is_current: entry.is_current,
            },
            section: Some(SectionType::Reverting),
        });
    }

    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_revert_in_progress_no_files() {
        // A fresh temp dir (not a real git repo) has no .git/REVERT_HEAD
        let dir = tempfile::tempdir().unwrap();
        assert!(!revert_in_progress(dir.path()));
    }

    #[test]
    fn test_revert_in_progress_with_revert_head() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("REVERT_HEAD"), "abc1234\n").unwrap();
        assert!(revert_in_progress(dir.path()));
    }

    #[test]
    fn test_revert_in_progress_with_sequencer_todo() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let sequencer = git_dir.join("sequencer");
        fs::create_dir_all(&sequencer).unwrap();
        fs::write(
            sequencer.join("todo"),
            "revert abc1234 # Some commit message\n",
        )
        .unwrap();
        assert!(revert_in_progress(dir.path()));
    }

    #[test]
    fn test_revert_in_progress_sequencer_not_revert() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let sequencer = git_dir.join("sequencer");
        fs::create_dir_all(&sequencer).unwrap();
        fs::write(
            sequencer.join("todo"),
            "pick abc1234 # Some commit message\n",
        )
        .unwrap();
        assert!(!revert_in_progress(dir.path()));
    }

    #[test]
    fn test_get_reverting_entries_with_revert_head() {
        let dir = tempfile::tempdir().unwrap();
        // Need a real git repo for this to work, so we skip the commit message lookup
        // Just test the hash parsing
        let git_dir = dir.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("REVERT_HEAD"), "abc1234abcdef\n").unwrap();

        let entries = get_reverting_entries(dir.path());
        // We get one entry (message will be empty since there's no real git repo)
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].hash, "abc1234"); // truncated to 7 chars
        assert!(entries[0].is_current);
    }

    #[test]
    fn test_get_reverting_entries_with_sequencer() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let sequencer = git_dir.join("sequencer");
        fs::create_dir_all(&sequencer).unwrap();
        fs::write(
            sequencer.join("todo"),
            "revert abc1234abcdef # Fix bug\nrevert def5678abcdef # Add feature\n",
        )
        .unwrap();

        let entries = get_reverting_entries(dir.path());
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].hash, "abc1234");
        assert_eq!(entries[0].message, "Fix bug");
        assert!(!entries[0].is_current);
        assert_eq!(entries[1].hash, "def5678");
        assert_eq!(entries[1].message, "Add feature");
        assert!(!entries[1].is_current);
    }

    #[test]
    fn test_get_reverting_lines_empty_when_not_in_progress() {
        let dir = tempfile::tempdir().unwrap();
        // No .git dir at all
        let lines = get_reverting_lines(dir.path()).unwrap();
        assert!(lines.is_empty());
    }

    // Helper for path existence (unused but documents expectations)
    fn _git_dir_path(workdir: &Path) -> PathBuf {
        workdir.join(".git")
    }
}
