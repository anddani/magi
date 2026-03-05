use std::path::Path;

use crate::git::git_cmd;
use crate::model::{Line, LineContent, PreviewLineType};

/// Parse raw `git show`/`git stash show` output into model Lines.
pub fn parse_preview_output(output: &str) -> Vec<Line> {
    let mut in_diff = false;
    output
        .lines()
        .map(|line| {
            if line.starts_with("diff --git ") {
                in_diff = true;
            }
            let line_type = if !in_diff {
                PreviewLineType::Header
            } else if line.starts_with("diff ")
                || line.starts_with("index ")
                || line.starts_with("--- ")
                || line.starts_with("+++ ")
                || line.starts_with("new file")
                || line.starts_with("deleted file")
                || line.starts_with("rename ")
                || line.starts_with("similarity ")
            {
                PreviewLineType::DiffFileHeader
            } else if line.starts_with("@@") {
                PreviewLineType::HunkHeader
            } else if line.starts_with('+') {
                PreviewLineType::Addition
            } else if line.starts_with('-') {
                PreviewLineType::Deletion
            } else {
                PreviewLineType::Context
            };
            Line {
                content: LineContent::PreviewLine {
                    content: line.to_string(),
                    line_type,
                },
                section: None,
            }
        })
        .collect()
}

/// Returns preview lines for a commit (runs `git show <hash>`).
pub fn get_commit_preview_lines(workdir: &Path, hash: &str) -> Vec<Line> {
    let output = git_cmd(workdir, &["show", hash])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    parse_preview_output(&output)
}

/// Returns preview lines for a stash entry (runs `git stash show -p stash@{N}`).
pub fn get_stash_preview_lines(workdir: &Path, stash_index: usize) -> Vec<Line> {
    let stash_ref = format!("stash@{{{stash_index}}}");
    let output = git_cmd(workdir, &["stash", "show", "-p", &stash_ref])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    parse_preview_output(&output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_output() {
        let lines = parse_preview_output("");
        assert!(lines.is_empty());
    }

    #[test]
    fn test_parse_header_lines() {
        let output = "commit abc1234\nAuthor: Test User <test@example.com>\nDate:   Mon Jan 1 00:00:00 2024\n\n    Add feature\n";
        let lines = parse_preview_output(output);
        assert!(!lines.is_empty());
        for line in &lines {
            if let LineContent::PreviewLine { line_type, .. } = &line.content {
                assert_eq!(*line_type, PreviewLineType::Header);
            }
        }
    }

    #[test]
    fn test_parse_diff_sections() {
        let output = "commit abc1234\nAuthor: Test\n\n    Message\n\ndiff --git a/foo.rs b/foo.rs\nindex 1234567..abcdefg 100644\n--- a/foo.rs\n+++ b/foo.rs\n@@ -1,3 +1,4 @@\n context\n+added line\n-removed line\n context2\n";
        let lines = parse_preview_output(output);

        let contents: Vec<(&str, &PreviewLineType)> = lines
            .iter()
            .filter_map(|l| {
                if let LineContent::PreviewLine { content, line_type } = &l.content {
                    Some((content.as_str(), line_type))
                } else {
                    None
                }
            })
            .collect();

        // First lines are headers
        assert!(matches!(contents[0].1, PreviewLineType::Header));

        // Find and verify diff file header
        let diff_line = contents
            .iter()
            .find(|(c, _)| c.starts_with("diff --git"))
            .unwrap();
        assert_eq!(*diff_line.1, PreviewLineType::DiffFileHeader);

        // Find and verify hunk header
        let hunk_line = contents.iter().find(|(c, _)| c.starts_with("@@")).unwrap();
        assert_eq!(*hunk_line.1, PreviewLineType::HunkHeader);

        // Find and verify addition
        let added = contents
            .iter()
            .find(|(c, _)| c.starts_with('+') && !c.starts_with("+++"))
            .unwrap();
        assert_eq!(*added.1, PreviewLineType::Addition);

        // Find and verify deletion
        let deleted = contents
            .iter()
            .find(|(c, _)| c.starts_with('-') && !c.starts_with("---"))
            .unwrap();
        assert_eq!(*deleted.1, PreviewLineType::Deletion);

        // Find context line (a line that is typed as Context)
        let ctx = contents
            .iter()
            .find(|(_, t)| matches!(*t, PreviewLineType::Context))
            .unwrap();
        assert_eq!(*ctx.1, PreviewLineType::Context);
    }

    #[test]
    fn test_parse_no_diff_all_headers() {
        // Output without any diff section — all lines should be Header
        let output = "commit abc1234\nAuthor: Test\n\n    Initial commit\n";
        let lines = parse_preview_output(output);
        for line in &lines {
            if let LineContent::PreviewLine { line_type, .. } = &line.content {
                assert_eq!(*line_type, PreviewLineType::Header);
            }
        }
    }
}
