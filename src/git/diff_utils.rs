use crate::{
    errors::MagiResult,
    model::{
        DiffHunk, DiffLine, DiffLineType, FileChange, FileStatus, Line, LineContent, SectionType,
    },
};
use git2::Diff;

/// A collection of file changes with their associated hunks and diff lines
pub type FileChangesWithDiffs = Vec<(FileChange, Vec<(DiffHunk, Vec<DiffLine>)>)>;

/// Collects file changes with their associated hunks and diff lines from a git diff
pub fn collect_file_changes(diff: &Diff) -> MagiResult<FileChangesWithDiffs> {
    let mut result: FileChangesWithDiffs = Vec::new();

    diff.print(git2::DiffFormat::Patch, |delta, hunk, line| {
        // Conflicted files are collected separately with their combined diff
        // (libgit2 emits them without hunk content)
        if delta.status() == git2::Delta::Conflicted {
            return true;
        }

        let file_path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "<unknown>".to_string());

        let status = match delta.status() {
            git2::Delta::Modified => FileStatus::Modified,
            git2::Delta::Deleted => FileStatus::Deleted,
            git2::Delta::Added => FileStatus::New,
            git2::Delta::Renamed => FileStatus::Renamed,
            git2::Delta::Copied => FileStatus::Copied,
            git2::Delta::Typechange => FileStatus::TypeChange,
            _ => FileStatus::Modified,
        };

        // Find or create the file entry
        let file_idx = result
            .iter()
            .position(|(fc, _)| fc.path == file_path)
            .unwrap_or_else(|| {
                result.push((
                    FileChange {
                        path: file_path.clone(),
                        status,
                    },
                    Vec::new(),
                ));
                result.len() - 1
            });

        // Handle hunk header
        if let Some(hunk_info) = hunk {
            let header = String::from_utf8_lossy(hunk_info.header())
                .trim_end_matches('\n')
                .to_string();

            // Check if this hunk already exists for this file
            let hunk_exists = result[file_idx].1.iter().any(|(h, _)| h.header == header);

            if !hunk_exists {
                let hunk_index = result[file_idx].1.len();
                result[file_idx]
                    .1
                    .push((DiffHunk { header, hunk_index }, Vec::new()));
            }
        }

        // Handle diff line content
        let content = String::from_utf8_lossy(line.content()).to_string();
        let content = content.trim_end_matches('\n').to_string();

        let line_type = match line.origin() {
            '+' => Some(DiffLineType::Addition),
            '-' => Some(DiffLineType::Deletion),
            ' ' => Some(DiffLineType::Context),
            _ => None,
        };

        if let Some(lt) = line_type
            && let Some((_, diff_lines)) = result[file_idx].1.last_mut()
        {
            diff_lines.push(DiffLine {
                content,
                line_type: lt,
            });
        }

        true
    })?;

    Ok(result)
}

/// Converts file changes into lines for display.
///
/// This function takes file changes and closures that create the appropriate
/// section types and line content for either staged or unstaged changes.
pub fn build_change_lines<F, G, H>(
    file_changes: FileChangesWithDiffs,
    header_title: &str,
    header_section: SectionType,
    make_file_content: F,
    make_file_section: G,
    make_hunk_section: H,
) -> Vec<Line>
where
    F: Fn(FileChange) -> LineContent,
    G: Fn(String) -> SectionType,
    H: Fn(String, usize) -> SectionType,
{
    let mut lines = Vec::new();
    let count = file_changes.len();

    if count == 0 {
        return lines;
    }

    // Add section header
    lines.push(Line {
        content: LineContent::SectionHeader {
            title: header_title.to_string(),
            count: Some(count),
        },
        section: Some(header_section),
    });

    // Add each file with its diff
    for (file_change, hunks) in file_changes {
        let file_path = file_change.path.clone();

        // Add file line
        lines.push(Line {
            content: make_file_content(file_change),
            section: Some(make_file_section(file_path.clone())),
        });

        // Add hunks and diff lines
        for (hunk_index, (hunk, diff_lines)) in hunks.into_iter().enumerate() {
            let hunk_section = make_hunk_section(file_path.clone(), hunk_index);

            // Add hunk header
            lines.push(Line {
                content: LineContent::DiffHunk(hunk),
                section: Some(hunk_section.clone()),
            });

            // Add diff lines
            for diff_line in diff_lines {
                lines.push(Line {
                    content: LineContent::DiffLine(diff_line),
                    section: Some(hunk_section.clone()),
                });
            }
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::test_repo::TestRepo;
    use git2::DiffOptions;

    fn open_repo(test_repo: &TestRepo) -> git2::Repository {
        git2::Repository::open(test_repo.repo_path()).unwrap()
    }

    /// Collects file changes from the index-to-workdir (unstaged) diff.
    fn unstaged_changes(repo: &git2::Repository) -> FileChangesWithDiffs {
        let mut opts = DiffOptions::new();
        opts.include_untracked(false);
        let diff = repo.diff_index_to_workdir(None, Some(&mut opts)).unwrap();
        collect_file_changes(&diff).unwrap()
    }

    /// Collects file changes from the HEAD-to-index (staged) diff.
    fn staged_changes(repo: &git2::Repository) -> FileChangesWithDiffs {
        let head = repo.head().unwrap().peel_to_tree().unwrap();
        let diff = repo.diff_tree_to_index(Some(&head), None, None).unwrap();
        collect_file_changes(&diff).unwrap()
    }

    /// Maps diff lines to (content, line_type) pairs for easy comparison.
    fn line_pairs(lines: &[DiffLine]) -> Vec<(String, DiffLineType)> {
        lines
            .iter()
            .map(|l| (l.content.clone(), l.line_type.clone()))
            .collect()
    }

    #[test]
    fn test_collect_file_changes_empty_diff() {
        let test_repo = TestRepo::new();
        let repo = open_repo(&test_repo);

        let changes = unstaged_changes(&repo);
        assert!(changes.is_empty());
    }

    #[test]
    fn test_collect_file_changes_modified_file_single_hunk() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("file.txt", "line1\nline2\nline3\n", "Add file");
        test_repo.write_file_content("file.txt", "line1\nCHANGED\nline3\n");

        let repo = open_repo(&test_repo);
        let changes = unstaged_changes(&repo);

        assert_eq!(changes.len(), 1);
        let (file_change, hunks) = &changes[0];
        assert_eq!(file_change.path, "file.txt");
        assert_eq!(file_change.status, FileStatus::Modified);

        assert_eq!(hunks.len(), 1);
        let (hunk, lines) = &hunks[0];
        assert!(hunk.header.starts_with("@@ -1,3 +1,3 @@"));
        assert_eq!(hunk.hunk_index, 0);

        assert_eq!(
            line_pairs(lines),
            vec![
                ("line1".to_string(), DiffLineType::Context),
                ("line2".to_string(), DiffLineType::Deletion),
                ("CHANGED".to_string(), DiffLineType::Addition),
                ("line3".to_string(), DiffLineType::Context),
            ]
        );
    }

    #[test]
    fn test_collect_file_changes_additions_only_new_file() {
        let test_repo = TestRepo::new();
        test_repo
            .write_file_content("new.txt", "one\ntwo\n")
            .stage_files(&["new.txt"]);

        let repo = open_repo(&test_repo);
        let changes = staged_changes(&repo);

        assert_eq!(changes.len(), 1);
        let (file_change, hunks) = &changes[0];
        assert_eq!(file_change.path, "new.txt");
        assert_eq!(file_change.status, FileStatus::New);

        assert_eq!(hunks.len(), 1);
        let (hunk, lines) = &hunks[0];
        assert!(hunk.header.starts_with("@@ -0,0 +1,2 @@"));

        assert_eq!(
            line_pairs(lines),
            vec![
                ("one".to_string(), DiffLineType::Addition),
                ("two".to_string(), DiffLineType::Addition),
            ]
        );
    }

    #[test]
    fn test_collect_file_changes_deletions_only_deleted_file() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("doomed.txt", "one\ntwo\n", "Add doomed file");
        test_repo
            .delete_file("doomed.txt")
            .stage_files(&["doomed.txt"]);

        let repo = open_repo(&test_repo);
        let changes = staged_changes(&repo);

        assert_eq!(changes.len(), 1);
        let (file_change, hunks) = &changes[0];
        assert_eq!(file_change.path, "doomed.txt");
        assert_eq!(file_change.status, FileStatus::Deleted);

        assert_eq!(hunks.len(), 1);
        let (hunk, lines) = &hunks[0];
        assert!(hunk.header.starts_with("@@ -1,2 +0,0 @@"));

        assert_eq!(
            line_pairs(lines),
            vec![
                ("one".to_string(), DiffLineType::Deletion),
                ("two".to_string(), DiffLineType::Deletion),
            ]
        );
    }

    #[test]
    fn test_collect_file_changes_multiple_hunks_indexed() {
        let test_repo = TestRepo::new();
        // 20 lines so that a change at the top and one at the bottom produce
        // two separate hunks (default context is 3 lines).
        let content = (1..=20)
            .map(|i| format!("line{:02}\n", i))
            .collect::<String>();
        test_repo.commit_file("file.txt", &content, "Add file");

        let modified = content
            .replace("line01\n", "first-changed\n")
            .replace("line20\n", "last-changed\n");
        test_repo.write_file_content("file.txt", &modified);

        let repo = open_repo(&test_repo);
        let changes = unstaged_changes(&repo);

        assert_eq!(changes.len(), 1);
        let (_, hunks) = &changes[0];
        assert_eq!(hunks.len(), 2);

        let (first_hunk, first_lines) = &hunks[0];
        assert_eq!(first_hunk.hunk_index, 0);
        assert!(first_hunk.header.starts_with("@@ -1,4 +1,4 @@"));
        assert!(
            first_lines
                .iter()
                .any(|l| l.content == "first-changed" && l.line_type == DiffLineType::Addition)
        );
        assert!(first_lines.iter().all(|l| l.content != "last-changed"));

        let (second_hunk, second_lines) = &hunks[1];
        assert_eq!(second_hunk.hunk_index, 1);
        assert!(second_hunk.header.starts_with("@@ -17,4 +17,4 @@"));
        assert!(
            second_lines
                .iter()
                .any(|l| l.content == "last-changed" && l.line_type == DiffLineType::Addition)
        );
        assert!(second_lines.iter().all(|l| l.content != "first-changed"));
    }

    #[test]
    fn test_collect_file_changes_multiple_files() {
        let test_repo = TestRepo::new();
        test_repo.commit_file("a.txt", "aaa\n", "Add a");
        test_repo.commit_file("b.txt", "bbb\n", "Add b");
        test_repo
            .write_file_content("a.txt", "aaa changed\n")
            .write_file_content("b.txt", "bbb changed\n");

        let repo = open_repo(&test_repo);
        let changes = unstaged_changes(&repo);

        assert_eq!(changes.len(), 2);

        for (path, expected_addition) in [("a.txt", "aaa changed"), ("b.txt", "bbb changed")] {
            let (file_change, hunks) = changes
                .iter()
                .find(|(fc, _)| fc.path == path)
                .unwrap_or_else(|| panic!("missing file change for {}", path));
            assert_eq!(file_change.status, FileStatus::Modified);
            assert_eq!(hunks.len(), 1);
            assert!(
                hunks[0].1.iter().any(
                    |l| l.content == expected_addition && l.line_type == DiffLineType::Addition
                ),
                "hunk for {} should contain its own addition",
                path
            );
        }
    }

    #[test]
    fn test_collect_file_changes_no_trailing_newline() {
        let test_repo = TestRepo::new();
        // Neither the old nor the new version ends with a newline.
        test_repo.commit_file("file.txt", "alpha\nbeta", "Add file");
        test_repo.write_file_content("file.txt", "alpha\ngamma");

        let repo = open_repo(&test_repo);
        let changes = unstaged_changes(&repo);

        assert_eq!(changes.len(), 1);
        let (_, hunks) = &changes[0];
        assert_eq!(hunks.len(), 1);

        // The "\ No newline at end of file" marker lines (origins '<', '>', '=')
        // must not show up as diff lines.
        assert_eq!(
            line_pairs(&hunks[0].1),
            vec![
                ("alpha".to_string(), DiffLineType::Context),
                ("beta".to_string(), DiffLineType::Deletion),
                ("gamma".to_string(), DiffLineType::Addition),
            ]
        );
    }

    #[test]
    fn test_build_change_lines_empty_input() {
        let lines = build_change_lines(
            Vec::new(),
            "Unstaged changes",
            SectionType::UnstagedChanges,
            LineContent::UnstagedFile,
            |path| SectionType::UnstagedFile { path },
            |path, hunk_index| SectionType::UnstagedHunk { path, hunk_index },
        );
        assert!(lines.is_empty());
    }

    #[test]
    fn test_build_change_lines_structure() {
        let file_changes: FileChangesWithDiffs = vec![(
            FileChange {
                path: "a.txt".to_string(),
                status: FileStatus::Modified,
            },
            vec![(
                DiffHunk {
                    header: "@@ -1,2 +1,2 @@".to_string(),
                    hunk_index: 0,
                },
                vec![DiffLine {
                    content: "new line".to_string(),
                    line_type: DiffLineType::Addition,
                }],
            )],
        )];

        let lines = build_change_lines(
            file_changes,
            "Unstaged changes",
            SectionType::UnstagedChanges,
            LineContent::UnstagedFile,
            |path| SectionType::UnstagedFile { path },
            |path, hunk_index| SectionType::UnstagedHunk { path, hunk_index },
        );

        assert_eq!(lines.len(), 4);

        // Section header with file count
        assert!(matches!(
            &lines[0].content,
            LineContent::SectionHeader { title, count: Some(1) } if title == "Unstaged changes"
        ));
        assert_eq!(lines[0].section, Some(SectionType::UnstagedChanges));

        // File line
        assert!(matches!(
            &lines[1].content,
            LineContent::UnstagedFile(fc) if fc.path == "a.txt"
        ));
        assert_eq!(
            lines[1].section,
            Some(SectionType::UnstagedFile {
                path: "a.txt".to_string()
            })
        );

        // Hunk header and diff line both belong to the hunk section
        let hunk_section = SectionType::UnstagedHunk {
            path: "a.txt".to_string(),
            hunk_index: 0,
        };
        assert!(matches!(
            &lines[2].content,
            LineContent::DiffHunk(h) if h.header == "@@ -1,2 +1,2 @@" && h.hunk_index == 0
        ));
        assert_eq!(lines[2].section, Some(hunk_section.clone()));
        assert!(matches!(
            &lines[3].content,
            LineContent::DiffLine(l) if l.content == "new line"
                && l.line_type == DiffLineType::Addition
        ));
        assert_eq!(lines[3].section, Some(hunk_section));
    }
}
