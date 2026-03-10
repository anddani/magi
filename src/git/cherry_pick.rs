use std::fs;
use std::path::Path;

use super::read_commit_message;
use crate::{
    errors::MagiResult,
    i18n,
    model::{LineContent, SectionType},
};

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

/// Returns model lines for the "Cherry Picking" section.
/// Returns an empty vec if no cherry-pick is in progress.
pub fn get_cherry_picking_lines(workdir: &Path) -> MagiResult<Vec<crate::model::Line>> {
    if !cherry_pick_in_progress(workdir) {
        return Ok(vec![]);
    }

    let git_dir = workdir.join(".git");
    let mut lines = Vec::new();

    // Section header
    lines.push(crate::model::Line {
        content: LineContent::SectionHeader {
            title: i18n::t().section_cherry_picking.to_string(),
            count: None,
        },
        section: Some(SectionType::CherryPicking),
    });

    // Current stopped commit from CHERRY_PICK_HEAD
    let cherry_pick_head_path = git_dir.join("CHERRY_PICK_HEAD");
    if let Ok(hash_raw) = fs::read_to_string(&cherry_pick_head_path) {
        let hash = hash_raw.trim().to_string();
        if !hash.is_empty() {
            let short_hash: String = hash.chars().take(7).collect();
            let message = read_commit_message(workdir, &hash).unwrap_or_default();
            lines.push(crate::model::Line {
                content: LineContent::CherryPickingEntry {
                    hash: short_hash,
                    message,
                    is_current: true,
                },
                section: Some(SectionType::CherryPicking),
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
            // Format: "pick <full-hash> # <message>"
            if let Some(rest) = line.strip_prefix("pick ") {
                let parts: Vec<&str> = rest.splitn(2, " # ").collect();
                let short_hash: String = parts[0].trim().chars().take(7).collect();
                let message = parts
                    .get(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                lines.push(crate::model::Line {
                    content: LineContent::CherryPickingEntry {
                        hash: short_hash,
                        message,
                        is_current: false,
                    },
                    section: Some(SectionType::CherryPicking),
                });
            }
        }
    }

    // Only return the section if we have at least one entry besides the header
    if lines.len() <= 1 {
        return Ok(vec![]);
    }

    Ok(lines)
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
