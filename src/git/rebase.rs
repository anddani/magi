use std::fs;
use std::path::Path;

use super::git_cmd;
use crate::{
    errors::MagiResult,
    git::{commit::get_commit_result, read_commit_message},
    i18n,
    model::{LineContent, SectionType},
};

pub use super::commit::CommitResult;

/// Returns true if a rebase sequence is currently in progress.
/// Checks for `rebase-merge/` directory (interactive) or `rebase-apply/onto` (patch-based).
pub fn rebase_in_progress(workdir: &Path) -> bool {
    let git_dir = workdir.join(".git");
    git_dir.join("rebase-merge").is_dir() || git_dir.join("rebase-apply").join("onto").exists()
}

/// A single entry shown in the "Rebasing" section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebasingEntry {
    pub hash: String,
    pub message: String,
    /// true = the commit currently stopped on (conflict); false = pending in todo
    pub is_current: bool,
}

/// Returns the list of rebasing entries when a rebase sequence is in progress.
/// The stopped commit (if any) comes first, followed by pending todo entries.
pub fn get_rebasing_entries(workdir: &Path) -> Vec<RebasingEntry> {
    let git_dir = workdir.join(".git");
    let merge_dir = git_dir.join("rebase-merge");
    let mut entries = Vec::new();

    if !merge_dir.is_dir() {
        return entries;
    }

    // Current stopped commit from stopped-sha
    let stopped_sha_path = merge_dir.join("stopped-sha");
    if let Ok(hash_raw) = fs::read_to_string(&stopped_sha_path) {
        let hash = hash_raw.trim().to_string();
        if !hash.is_empty() {
            let short_hash: String = hash.chars().take(7).collect();
            let message = read_commit_message(workdir, &hash).unwrap_or_default();
            entries.push(RebasingEntry {
                hash: short_hash,
                message,
                is_current: true,
            });
        }
    }

    // Pending commits from git-rebase-todo
    let todo_path = merge_dir.join("git-rebase-todo");
    if let Ok(content) = fs::read_to_string(&todo_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Format: "<cmd> <hash> <message>"
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() < 2 {
                continue;
            }
            // Only show commit-based commands; skip exec/label/reset/merge/break
            match parts[0] {
                "pick" | "p" | "edit" | "e" | "reword" | "r" | "squash" | "s" | "fixup" | "f"
                | "drop" | "d" => {}
                _ => continue,
            }
            let short_hash: String = parts[1].chars().take(7).collect();
            let message = parts
                .get(2)
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            entries.push(RebasingEntry {
                hash: short_hash,
                message,
                is_current: false,
            });
        }
    }

    entries
}

/// Returns model lines for the "Rebasing" section.
/// Returns an empty vec if no rebase is in progress.
pub fn get_rebasing_lines(workdir: &Path) -> MagiResult<Vec<crate::model::Line>> {
    if !rebase_in_progress(workdir) {
        return Ok(vec![]);
    }

    let entries = get_rebasing_entries(workdir);
    if entries.is_empty() {
        return Ok(vec![]);
    }

    let mut lines = Vec::new();

    lines.push(crate::model::Line {
        content: LineContent::SectionHeader {
            title: i18n::t().section_rebasing.to_string(),
            count: None,
        },
        section: Some(SectionType::Rebasing),
    });

    for entry in entries {
        lines.push(crate::model::Line {
            content: LineContent::RebasingEntry {
                hash: entry.hash,
                message: entry.message,
                is_current: entry.is_current,
            },
            section: Some(SectionType::Rebasing),
        });
    }

    Ok(lines)
}

/// Runs `git rebase --continue` which opens the user's configured editor
/// to edit the commit message after resolving conflicts.
pub fn run_rebase_continue_with_editor<P: AsRef<Path>>(repo_path: P) -> MagiResult<CommitResult> {
    let status = git_cmd(&repo_path, &["rebase", "--continue"]).status()?;

    get_commit_result(repo_path, status, "Rebase continue")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rebase_in_progress_no_files() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!rebase_in_progress(dir.path()));
    }

    #[test]
    fn test_rebase_in_progress_with_rebase_merge_dir() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let rebase_merge = git_dir.join("rebase-merge");
        fs::create_dir_all(&rebase_merge).unwrap();
        assert!(rebase_in_progress(dir.path()));
    }

    #[test]
    fn test_rebase_in_progress_with_rebase_apply_onto() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let rebase_apply = git_dir.join("rebase-apply");
        fs::create_dir_all(&rebase_apply).unwrap();
        fs::write(rebase_apply.join("onto"), "abc1234\n").unwrap();
        assert!(rebase_in_progress(dir.path()));
    }

    #[test]
    fn test_rebase_not_in_progress_without_rebase_apply_onto() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        // Create rebase-apply dir but without "onto" file
        let rebase_apply = git_dir.join("rebase-apply");
        fs::create_dir_all(&rebase_apply).unwrap();
        assert!(!rebase_in_progress(dir.path()));
    }

    #[test]
    fn test_get_rebasing_entries_with_stopped_sha() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let rebase_merge = git_dir.join("rebase-merge");
        fs::create_dir_all(&rebase_merge).unwrap();
        fs::write(rebase_merge.join("stopped-sha"), "abc1234abcdef\n").unwrap();

        let entries = get_rebasing_entries(dir.path());
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].hash, "abc1234");
        assert!(entries[0].is_current);
    }

    #[test]
    fn test_get_rebasing_entries_with_todo() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let rebase_merge = git_dir.join("rebase-merge");
        fs::create_dir_all(&rebase_merge).unwrap();
        fs::write(
            rebase_merge.join("git-rebase-todo"),
            "pick abc1234abcdef Fix bug\npick def5678abcdef Add feature\n# comment line\n",
        )
        .unwrap();

        let entries = get_rebasing_entries(dir.path());
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].hash, "abc1234");
        assert_eq!(entries[0].message, "Fix bug");
        assert!(!entries[0].is_current);
        assert_eq!(entries[1].hash, "def5678");
        assert_eq!(entries[1].message, "Add feature");
        assert!(!entries[1].is_current);
    }

    #[test]
    fn test_get_rebasing_entries_skips_non_commit_commands() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let rebase_merge = git_dir.join("rebase-merge");
        fs::create_dir_all(&rebase_merge).unwrap();
        fs::write(
            rebase_merge.join("git-rebase-todo"),
            "pick abc1234abcdef Fix bug\nexec echo hello\nlabel my-label\npick def5678abcdef Add feature\n",
        )
        .unwrap();

        let entries = get_rebasing_entries(dir.path());
        // Only commit-based entries
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].hash, "abc1234");
        assert_eq!(entries[1].hash, "def5678");
    }

    #[test]
    fn test_get_rebasing_entries_stopped_and_todo() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let rebase_merge = git_dir.join("rebase-merge");
        fs::create_dir_all(&rebase_merge).unwrap();
        fs::write(rebase_merge.join("stopped-sha"), "aaa1111abcdef\n").unwrap();
        fs::write(
            rebase_merge.join("git-rebase-todo"),
            "pick bbb2222abcdef Next commit\n",
        )
        .unwrap();

        let entries = get_rebasing_entries(dir.path());
        assert_eq!(entries.len(), 2);
        assert!(entries[0].is_current);
        assert_eq!(entries[0].hash, "aaa1111");
        assert!(!entries[1].is_current);
        assert_eq!(entries[1].hash, "bbb2222");
    }

    #[test]
    fn test_get_rebasing_lines_empty_when_not_in_progress() {
        let dir = tempfile::tempdir().unwrap();
        let lines = get_rebasing_lines(dir.path()).unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn test_get_rebasing_lines_returns_section_and_entries() {
        let dir = tempfile::tempdir().unwrap();
        let git_dir = dir.path().join(".git");
        let rebase_merge = git_dir.join("rebase-merge");
        fs::create_dir_all(&rebase_merge).unwrap();
        fs::write(rebase_merge.join("stopped-sha"), "abc1234abcdef\n").unwrap();

        let lines = get_rebasing_lines(dir.path()).unwrap();
        // Section header + 1 entry
        assert_eq!(lines.len(), 2);
        assert!(matches!(
            &lines[0].content,
            crate::model::LineContent::SectionHeader { title, .. } if title == "Rebasing"
        ));
        assert!(matches!(
            &lines[1].content,
            crate::model::LineContent::RebasingEntry {
                is_current: true,
                ..
            }
        ));
    }
}
