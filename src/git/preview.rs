use std::path::Path;

use crate::errors::{MagiError, MagiResult};
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

/// Returns preview lines showing what merging `branch` into `head_name`
/// (the checked-out branch) would produce, without touching the working
/// tree or the index. Builds the merge result with
/// `git merge-tree --write-tree HEAD <branch>` and diffs it against HEAD.
/// When the merge would conflict, the diff shows the conflict markers.
pub fn get_merge_preview_lines(
    workdir: &Path,
    branch: &str,
    head_name: &str,
) -> MagiResult<Vec<Line>> {
    let output = git_cmd(workdir, &["merge-tree", "--write-tree", "HEAD", branch]).output()?;
    // Exit code 0 = clean merge, 1 = conflicts; both print the resulting
    // tree id on the first line. Anything else is a real error.
    let has_conflicts = output.status.code() == Some(1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let tree_id = stdout.lines().next().unwrap_or("").trim().to_string();
    if tree_id.is_empty() || !(output.status.success() || has_conflicts) {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(MagiError::Generic(format!(
            "Cannot preview merge of '{}': {}",
            branch,
            stderr.trim()
        )));
    }

    let diff_output = git_cmd(workdir, &["diff", "HEAD", &tree_id]).output()?;
    if !diff_output.status.success() {
        let stderr = String::from_utf8_lossy(&diff_output.stderr);
        return Err(MagiError::Generic(format!(
            "Cannot preview merge of '{}': {}",
            branch,
            stderr.trim()
        )));
    }
    let diff = String::from_utf8_lossy(&diff_output.stdout);

    let header_line = |content: String| Line {
        content: LineContent::PreviewLine {
            content,
            line_type: PreviewLineType::Header,
        },
        section: None,
    };

    let mut lines = vec![header_line(format!(
        "Preview merge of {} into {}",
        branch, head_name
    ))];
    if has_conflicts {
        lines.push(header_line(
            "Merging would produce conflicts (conflict markers shown below)".to_string(),
        ));
    }
    lines.push(header_line(String::new()));
    if diff.trim().is_empty() {
        lines.push(header_line(format!(
            "Merging '{}' would not introduce any changes",
            branch
        )));
    } else {
        lines.extend(parse_preview_output(&diff));
    }
    Ok(lines)
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
    use crate::git::test_repo::TestRepo;

    fn run_git(test_repo: &TestRepo, args: &[&str]) -> std::process::Output {
        git_cmd(test_repo.repo_path(), args).output().unwrap()
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

    fn preview_contents(lines: &[Line]) -> Vec<(&str, &PreviewLineType)> {
        lines
            .iter()
            .filter_map(|l| {
                if let LineContent::PreviewLine { content, line_type } = &l.content {
                    Some((content.as_str(), line_type))
                } else {
                    None
                }
            })
            .collect()
    }

    #[test]
    fn test_merge_preview_clean_merge_shows_incoming_changes() {
        let test_repo = TestRepo::new();
        setup_divergent_branches(
            &test_repo,
            ("main.txt", "main content\n"),
            ("feature.txt", "feature content\n"),
        );

        let lines = get_merge_preview_lines(test_repo.repo_path(), "feature", "main").unwrap();
        let contents = preview_contents(&lines);

        assert_eq!(contents[0].0, "Preview merge of feature into main");
        assert_eq!(*contents[0].1, PreviewLineType::Header);
        // The incoming feature file shows up as an addition.
        assert!(
            contents
                .iter()
                .any(|(c, t)| *c == "+feature content" && **t == PreviewLineType::Addition),
            "expected feature.txt content as addition: {:?}",
            contents
        );
        // Changes already on main are part of both sides and must not appear.
        assert!(
            !contents.iter().any(|(c, _)| c.contains("main content")),
            "main-only content should not be part of the preview: {:?}",
            contents
        );
        // No conflict note for a clean merge.
        assert!(!contents.iter().any(|(c, _)| c.contains("conflicts")));
    }

    #[test]
    fn test_merge_preview_conflicting_merge_shows_conflict_note_and_markers() {
        let test_repo = TestRepo::new();
        setup_divergent_branches(
            &test_repo,
            ("base.txt", "main change\n"),
            ("base.txt", "feature change\n"),
        );

        let lines = get_merge_preview_lines(test_repo.repo_path(), "feature", "main").unwrap();
        let contents = preview_contents(&lines);

        assert!(
            contents
                .iter()
                .any(|(c, t)| c.contains("conflicts") && **t == PreviewLineType::Header),
            "expected a conflict note header: {:?}",
            contents
        );
        assert!(
            contents.iter().any(|(c, _)| c.starts_with("+<<<<<<<")),
            "expected conflict markers in the diff: {:?}",
            contents
        );
    }

    #[test]
    fn test_merge_preview_does_not_touch_worktree_or_start_merge() {
        let test_repo = TestRepo::new();
        setup_divergent_branches(
            &test_repo,
            ("main.txt", "main content\n"),
            ("feature.txt", "feature content\n"),
        );

        get_merge_preview_lines(test_repo.repo_path(), "feature", "main").unwrap();

        assert!(!test_repo.repo_path().join("feature.txt").exists());
        assert!(!test_repo.repo.path().join("MERGE_HEAD").exists());
        let status = run_git(&test_repo, &["status", "--porcelain"]);
        assert!(String::from_utf8_lossy(&status.stdout).trim().is_empty());
    }

    #[test]
    fn test_merge_preview_already_merged_branch_reports_no_changes() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("base.txt", "base\n", "Base commit");
        test_repo.create_branch("feature");
        test_repo.commit_file("main.txt", "main content\n", "Main commit");

        // `feature` is an ancestor of `main`: merging changes nothing.
        let lines = get_merge_preview_lines(test_repo.repo_path(), "feature", "main").unwrap();
        let contents = preview_contents(&lines);

        assert!(
            contents
                .iter()
                .any(|(c, _)| c.contains("would not introduce any changes")),
            "expected a no-changes note: {:?}",
            contents
        );
    }

    #[test]
    fn test_merge_preview_unknown_branch_fails() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("base.txt", "base\n", "Base commit");

        let result = get_merge_preview_lines(test_repo.repo_path(), "no-such-branch", "main");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cannot preview merge of 'no-such-branch'")
        );
    }

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
