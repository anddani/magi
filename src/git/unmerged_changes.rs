use git2::Repository;

use crate::{
    errors::MagiResult,
    model::{DiffHunk, DiffLine, DiffLineType, FileChange, FileStatus},
};

use super::{diff_utils::FileChangesWithDiffs, git_cmd};

/// Collects unmerged (conflicted) files with their combined diffs.
///
/// libgit2 reports conflicted files as `Delta::Conflicted` without any hunk
/// content, so the combined diff (`diff --cc`) is read from the git CLI
/// instead, preserving the conflict markers in the workdir file.
pub fn collect_unmerged_changes(repository: &Repository) -> MagiResult<FileChangesWithDiffs> {
    let mut status_options = git2::StatusOptions::new();
    status_options.include_untracked(false);
    status_options.include_ignored(false);

    let statuses = repository.statuses(Some(&mut status_options))?;

    let conflicted_paths: Vec<String> = statuses
        .iter()
        .filter_map(|entry| {
            if entry.status().is_conflicted() {
                entry.path().ok().map(|path| path.to_string())
            } else {
                None
            }
        })
        .collect();

    let workdir = repository
        .workdir()
        .unwrap_or_else(|| std::path::Path::new("."));

    let mut result: FileChangesWithDiffs = Vec::new();
    for path in conflicted_paths {
        let output = git_cmd(workdir, &["diff", "--", &path]).output()?;
        let diff_text = String::from_utf8_lossy(&output.stdout);
        let hunks = parse_combined_diff(&diff_text);

        result.push((
            FileChange {
                path,
                status: FileStatus::Unmerged,
            },
            hunks,
        ));
    }

    Ok(result)
}

/// Parses the combined diff format that `git diff` produces for unmerged
/// files. Hunk headers look like `@@@ -1,3 -1,3 +1,7 @@@` and each line
/// carries one origin column per parent (two for a normal merge), e.g.
/// `++<<<<<<< HEAD` or `+ ours`.
fn parse_combined_diff(diff_text: &str) -> Vec<(DiffHunk, Vec<DiffLine>)> {
    let mut hunks: Vec<(DiffHunk, Vec<DiffLine>)> = Vec::new();
    // One origin column per parent: "@@@" (3 x '@') means 2 parents
    let mut prefix_width = 2;

    for line in diff_text.lines() {
        if line.starts_with("@@@") {
            prefix_width = line.chars().take_while(|&c| c == '@').count() - 1;
            hunks.push((
                DiffHunk {
                    header: line.to_string(),
                    hunk_index: hunks.len(),
                },
                Vec::new(),
            ));
            continue;
        }

        let Some((_, diff_lines)) = hunks.last_mut() else {
            // Still in the preamble (diff --cc, index, ---/+++ lines)
            continue;
        };

        // Skip "\ No newline at end of file" markers
        if line.starts_with('\\') {
            continue;
        }

        let prefix: String = line.chars().take(prefix_width).collect();
        let rest: String = line.chars().skip(prefix_width).collect();

        let is_conflict_marker = ["<<<<<<<", "|||||||", "=======", ">>>>>>>"]
            .iter()
            .any(|marker| rest.starts_with(marker));

        let line_type = if is_conflict_marker {
            DiffLineType::ConflictMarker
        } else if prefix.contains('+') {
            DiffLineType::CombinedAddition
        } else if prefix.contains('-') {
            DiffLineType::CombinedDeletion
        } else {
            DiffLineType::CombinedContext
        };

        diff_lines.push(DiffLine {
            content: line.to_string(),
            line_type,
        });
    }

    hunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;

    #[test]
    fn test_parse_combined_diff_conflict() {
        let diff_text = "\
diff --cc file.txt
index 1234567,89abcde..0000000
--- a/file.txt
+++ b/file.txt
@@@ -1,1 -1,1 +1,5 @@@
++<<<<<<< HEAD
 +main content
++=======
+ other content
++>>>>>>> other
  shared line
";
        let hunks = parse_combined_diff(diff_text);

        assert_eq!(hunks.len(), 1);
        let (hunk, lines) = &hunks[0];
        assert_eq!(hunk.header, "@@@ -1,1 -1,1 +1,5 @@@");
        assert_eq!(hunk.hunk_index, 0);

        let pairs: Vec<(&str, &DiffLineType)> = lines
            .iter()
            .map(|l| (l.content.as_str(), &l.line_type))
            .collect();
        assert_eq!(
            pairs,
            vec![
                ("++<<<<<<< HEAD", &DiffLineType::ConflictMarker),
                (" +main content", &DiffLineType::CombinedAddition),
                ("++=======", &DiffLineType::ConflictMarker),
                ("+ other content", &DiffLineType::CombinedAddition),
                ("++>>>>>>> other", &DiffLineType::ConflictMarker),
                ("  shared line", &DiffLineType::CombinedContext),
            ]
        );
    }

    #[test]
    fn test_parse_combined_diff_deletion_line() {
        let diff_text = "\
@@@ -1,2 -1,2 +1,1 @@@
- removed in ours
  kept line
";
        let hunks = parse_combined_diff(diff_text);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].1[0].line_type, DiffLineType::CombinedDeletion);
        assert_eq!(hunks[0].1[1].line_type, DiffLineType::CombinedContext);
    }

    #[test]
    fn test_parse_combined_diff_empty_input() {
        assert!(parse_combined_diff("").is_empty());
    }

    #[test]
    fn test_collect_unmerged_changes_no_conflicts() {
        let test_repo = TestRepo::new();
        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        assert!(collect_unmerged_changes(&repo).unwrap().is_empty());
    }

    #[test]
    fn test_collect_unmerged_changes_merge_conflict() {
        let test_repo = TestRepo::new();
        test_repo.create_merge_conflict("conflict.txt");

        let repo = git2::Repository::open(test_repo.repo_path()).unwrap();
        let changes = collect_unmerged_changes(&repo).unwrap();

        assert_eq!(changes.len(), 1);
        let (file_change, hunks) = &changes[0];
        assert_eq!(file_change.path, "conflict.txt");
        assert_eq!(file_change.status, FileStatus::Unmerged);

        assert_eq!(hunks.len(), 1);
        let (hunk, lines) = &hunks[0];
        assert!(hunk.header.starts_with("@@@"));
        assert!(
            lines
                .iter()
                .any(|l| l.content.contains("<<<<<<<")
                    && l.line_type == DiffLineType::ConflictMarker)
        );
        assert!(
            lines
                .iter()
                .any(|l| l.content.contains(">>>>>>>")
                    && l.line_type == DiffLineType::ConflictMarker)
        );
    }
}
