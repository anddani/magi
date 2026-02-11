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
            let header = String::from_utf8_lossy(hunk_info.header()).to_string();

            // Check if this hunk already exists for this file
            let hunk_exists = result[file_idx].1.iter().any(|(h, _)| h.header == header);

            if !hunk_exists {
                result[file_idx].1.push((DiffHunk { header }, Vec::new()));
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
