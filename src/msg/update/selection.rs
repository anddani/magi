//! Shared selection logic for stage/unstage operations.
//!
//! This module provides a unified way to determine what is selected in the UI,
//! whether in normal mode (cursor position) or visual mode (selection range).

use std::collections::HashSet;

use crate::model::{Line, LineContent, SectionType};

/// The context for selection - whether we're looking at staged or unstaged content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionContext {
    /// Looking for content that can be staged (untracked files, unstaged changes)
    Stageable,
    /// Looking for content that can be unstaged (staged changes)
    Unstageable,
    /// Looking for content that can be discarded (untracked, unstaged, or staged changes)
    Discardable,
}

/// Represents what is currently selected in the UI.
#[derive(Debug, Clone)]
pub enum Selection<'a> {
    /// No actionable selection
    None,
    /// A list of file paths (for staging/unstaging entire files)
    Files(Vec<&'a str>),
    /// A single hunk identified by file path and hunk index
    Hunk { path: &'a str, hunk_index: usize },
    /// Multiple hunks in the same file (indices in reverse order for safe application)
    Hunks {
        path: &'a str,
        hunk_indices: Vec<usize>,
    },
    /// Specific lines within a hunk
    Lines {
        path: &'a str,
        hunk_index: usize,
        line_indices: Vec<usize>,
    },
}

/// Determines what is selected based on cursor position (normal mode).
pub fn get_normal_mode_selection<'a>(
    lines: &'a [Line],
    cursor_position: usize,
    context: SelectionContext,
) -> Selection<'a> {
    if cursor_position >= lines.len() {
        return Selection::None;
    }

    let line = &lines[cursor_position];

    match context {
        SelectionContext::Stageable => get_stageable_normal_selection(lines, line),
        SelectionContext::Unstageable => get_unstageable_normal_selection(lines, line),
        SelectionContext::Discardable => get_discardable_normal_selection(lines, line),
    }
}

fn get_stageable_normal_selection<'a>(lines: &'a [Line], line: &'a Line) -> Selection<'a> {
    match (&line.content, &line.section) {
        // Section header for untracked files → all untracked files
        (LineContent::SectionHeader { .. }, Some(SectionType::UntrackedFiles)) => {
            let files: Vec<&str> = lines
                .iter()
                .filter_map(|l| match &l.content {
                    LineContent::UntrackedFile(path) => Some(path.as_str()),
                    _ => None,
                })
                .collect();
            if files.is_empty() {
                Selection::None
            } else {
                Selection::Files(files)
            }
        }
        // Section header for unstaged changes → all unstaged files
        (LineContent::SectionHeader { .. }, Some(SectionType::UnstagedChanges)) => {
            let files: Vec<&str> = lines
                .iter()
                .filter_map(|l| match &l.content {
                    LineContent::UnstagedFile(fc) => Some(fc.path.as_str()),
                    _ => None,
                })
                .collect();
            if files.is_empty() {
                Selection::None
            } else {
                Selection::Files(files)
            }
        }
        // Untracked file → that file
        (LineContent::UntrackedFile(path), _) => Selection::Files(vec![path.as_str()]),
        // Unstaged file → that file
        (LineContent::UnstagedFile(fc), _) => Selection::Files(vec![fc.path.as_str()]),
        // Diff hunk in unstaged section → that hunk
        (LineContent::DiffHunk(_), Some(SectionType::UnstagedHunk { path, hunk_index })) => {
            Selection::Hunk {
                path,
                hunk_index: *hunk_index,
            }
        }
        // Diff line in unstaged section → the whole hunk
        (LineContent::DiffLine(_), Some(SectionType::UnstagedHunk { path, hunk_index })) => {
            Selection::Hunk {
                path,
                hunk_index: *hunk_index,
            }
        }
        _ => Selection::None,
    }
}

fn get_unstageable_normal_selection<'a>(lines: &'a [Line], line: &'a Line) -> Selection<'a> {
    match (&line.content, &line.section) {
        // Section header for staged changes → all staged files
        (LineContent::SectionHeader { .. }, Some(SectionType::StagedChanges)) => {
            let files: Vec<&str> = lines
                .iter()
                .filter_map(|l| match &l.content {
                    LineContent::StagedFile(fc) => Some(fc.path.as_str()),
                    _ => None,
                })
                .collect();
            if files.is_empty() {
                Selection::None
            } else {
                Selection::Files(files)
            }
        }
        // Staged file → that file
        (LineContent::StagedFile(fc), _) => Selection::Files(vec![fc.path.as_str()]),
        // Diff hunk in staged section → that hunk
        (LineContent::DiffHunk(_), Some(SectionType::StagedHunk { path, hunk_index })) => {
            Selection::Hunk {
                path,
                hunk_index: *hunk_index,
            }
        }
        // Diff line in staged section → the whole hunk
        (LineContent::DiffLine(_), Some(SectionType::StagedHunk { path, hunk_index })) => {
            Selection::Hunk {
                path,
                hunk_index: *hunk_index,
            }
        }
        _ => Selection::None,
    }
}

fn get_discardable_normal_selection<'a>(lines: &'a [Line], line: &'a Line) -> Selection<'a> {
    match (&line.content, &line.section) {
        // Section header for untracked files → all untracked files
        (LineContent::SectionHeader { .. }, Some(SectionType::UntrackedFiles)) => {
            let files: Vec<&str> = lines
                .iter()
                .filter_map(|l| match &l.content {
                    LineContent::UntrackedFile(path) => Some(path.as_str()),
                    _ => None,
                })
                .collect();
            if files.is_empty() {
                Selection::None
            } else {
                Selection::Files(files)
            }
        }
        // Untracked file → that file
        (LineContent::UntrackedFile(path), _) => Selection::Files(vec![path.as_str()]),
        // Section header for unstaged changes → all unstaged files
        (LineContent::SectionHeader { .. }, Some(SectionType::UnstagedChanges)) => {
            let files: Vec<&str> = lines
                .iter()
                .filter_map(|l| match &l.content {
                    LineContent::UnstagedFile(fc) => Some(fc.path.as_str()),
                    _ => None,
                })
                .collect();
            if files.is_empty() {
                Selection::None
            } else {
                Selection::Files(files)
            }
        }
        // Section header for staged changes → all staged files
        (LineContent::SectionHeader { .. }, Some(SectionType::StagedChanges)) => {
            let files: Vec<&str> = lines
                .iter()
                .filter_map(|l| match &l.content {
                    LineContent::StagedFile(fc) => Some(fc.path.as_str()),
                    _ => None,
                })
                .collect();
            if files.is_empty() {
                Selection::None
            } else {
                Selection::Files(files)
            }
        }
        // Unstaged file → that file
        (LineContent::UnstagedFile(fc), _) => Selection::Files(vec![fc.path.as_str()]),
        // Staged file → that file
        (LineContent::StagedFile(fc), _) => Selection::Files(vec![fc.path.as_str()]),
        // Diff hunk in unstaged section → that hunk
        (LineContent::DiffHunk(_), Some(SectionType::UnstagedHunk { path, hunk_index })) => {
            Selection::Hunk {
                path,
                hunk_index: *hunk_index,
            }
        }
        // Diff line in unstaged section → the whole hunk
        (LineContent::DiffLine(_), Some(SectionType::UnstagedHunk { path, hunk_index })) => {
            Selection::Hunk {
                path,
                hunk_index: *hunk_index,
            }
        }
        // Diff hunk in staged section → that hunk
        (LineContent::DiffHunk(_), Some(SectionType::StagedHunk { path, hunk_index })) => {
            Selection::Hunk {
                path,
                hunk_index: *hunk_index,
            }
        }
        // Diff line in staged section → the whole hunk
        (LineContent::DiffLine(_), Some(SectionType::StagedHunk { path, hunk_index })) => {
            Selection::Hunk {
                path,
                hunk_index: *hunk_index,
            }
        }
        _ => Selection::None,
    }
}

/// Determines what is selected based on visual selection range.
pub fn get_visual_mode_selection<'a>(
    lines: &'a [Line],
    start: usize,
    end: usize,
    collapsed_sections: &HashSet<SectionType>,
    context: SelectionContext,
) -> Selection<'a> {
    let end = end.min(lines.len().saturating_sub(1));

    // Collect meaningful lines in selection (skip empty and hidden/collapsed lines)
    let selected: Vec<(usize, &Line)> = (start..=end)
        .filter_map(|i| {
            let line = &lines[i];
            if matches!(line.content, LineContent::EmptyLine) || line.is_hidden(collapsed_sections)
            {
                return None;
            }
            Some((i, line))
        })
        .collect();

    if selected.is_empty() {
        return Selection::None;
    }

    match context {
        SelectionContext::Stageable => get_stageable_visual_selection(lines, &selected, start, end),
        SelectionContext::Unstageable => {
            get_unstageable_visual_selection(lines, &selected, start, end)
        }
        SelectionContext::Discardable => {
            get_discardable_visual_selection(lines, &selected, start, end)
        }
    }
}

fn get_stageable_visual_selection<'a>(
    lines: &'a [Line],
    selected: &[(usize, &'a Line)],
    sel_start: usize,
    sel_end: usize,
) -> Selection<'a> {
    // Check if all are untracked files
    let all_untracked = selected.iter().all(|(_, l)| {
        matches!(
            l.content,
            LineContent::UntrackedFile(_) | LineContent::SectionHeader { .. }
        )
    });
    if all_untracked {
        let files: Vec<&str> = selected
            .iter()
            .filter_map(|(_, l)| match &l.content {
                LineContent::UntrackedFile(path) => Some(path.as_str()),
                _ => None,
            })
            .collect();
        if !files.is_empty() {
            return Selection::Files(files);
        }
    }

    // Check if all are unstaged files
    let all_unstaged_files = selected.iter().all(|(_, l)| {
        matches!(
            l.content,
            LineContent::UnstagedFile(_) | LineContent::SectionHeader { .. }
        )
    });
    if all_unstaged_files {
        let files: Vec<&str> = selected
            .iter()
            .filter_map(|(_, l)| match &l.content {
                LineContent::UnstagedFile(fc) => Some(fc.path.as_str()),
                _ => None,
            })
            .collect();
        if !files.is_empty() {
            return Selection::Files(files);
        }
    }

    // Check if all are diff lines in the same unstaged hunk
    let all_diff_lines = selected
        .iter()
        .all(|(_, l)| matches!(l.content, LineContent::DiffLine(_)));
    if all_diff_lines
        && let Some(selection) =
            try_get_lines_in_same_hunk(lines, selected, sel_start, sel_end, |s| {
                matches!(s, SectionType::UnstagedHunk { .. })
            })
    {
        return selection;
    }

    // Check if all are diff hunks (or diff lines within hunks) in the same unstaged file
    let all_hunks = selected.iter().all(|(_, l)| {
        matches!(
            l.content,
            LineContent::DiffHunk(_) | LineContent::DiffLine(_)
        )
    });
    if all_hunks
        && let Some(selection) = try_get_hunks_in_same_file(selected, |s| match s {
            SectionType::UnstagedHunk { path, hunk_index } => Some((path.as_str(), *hunk_index)),
            _ => None,
        })
    {
        return selection;
    }

    Selection::None
}

fn get_unstageable_visual_selection<'a>(
    lines: &'a [Line],
    selected: &[(usize, &'a Line)],
    sel_start: usize,
    sel_end: usize,
) -> Selection<'a> {
    // Check if all are staged files
    let all_staged_files = selected.iter().all(|(_, l)| {
        matches!(
            l.content,
            LineContent::StagedFile(_) | LineContent::SectionHeader { .. }
        )
    });
    if all_staged_files {
        let files: Vec<&str> = selected
            .iter()
            .filter_map(|(_, l)| match &l.content {
                LineContent::StagedFile(fc) => Some(fc.path.as_str()),
                _ => None,
            })
            .collect();
        if !files.is_empty() {
            return Selection::Files(files);
        }
    }

    // Check if all are diff lines in the same staged hunk
    let all_diff_lines = selected
        .iter()
        .all(|(_, l)| matches!(l.content, LineContent::DiffLine(_)));
    if all_diff_lines
        && let Some(selection) =
            try_get_lines_in_same_hunk(lines, selected, sel_start, sel_end, |s| {
                matches!(s, SectionType::StagedHunk { .. })
            })
    {
        return selection;
    }

    // Check if all are diff hunks (or diff lines within hunks) in the same staged file
    let all_hunks = selected.iter().all(|(_, l)| {
        matches!(
            l.content,
            LineContent::DiffHunk(_) | LineContent::DiffLine(_)
        )
    });
    if all_hunks
        && let Some(selection) = try_get_hunks_in_same_file(selected, |s| match s {
            SectionType::StagedHunk { path, hunk_index } => Some((path.as_str(), *hunk_index)),
            _ => None,
        })
    {
        return selection;
    }

    Selection::None
}

fn get_discardable_visual_selection<'a>(
    lines: &'a [Line],
    selected: &[(usize, &'a Line)],
    sel_start: usize,
    sel_end: usize,
) -> Selection<'a> {
    // Check if all are untracked files
    let all_untracked = selected.iter().all(|(_, l)| {
        matches!(
            l.content,
            LineContent::UntrackedFile(_) | LineContent::SectionHeader { .. }
        )
    });
    if all_untracked {
        let files: Vec<&str> = selected
            .iter()
            .filter_map(|(_, l)| match &l.content {
                LineContent::UntrackedFile(path) => Some(path.as_str()),
                _ => None,
            })
            .collect();
        if !files.is_empty() {
            return Selection::Files(files);
        }
    }

    // Check if all are unstaged files
    let all_unstaged_files = selected.iter().all(|(_, l)| {
        matches!(
            l.content,
            LineContent::UnstagedFile(_) | LineContent::SectionHeader { .. }
        )
    });
    if all_unstaged_files {
        let files: Vec<&str> = selected
            .iter()
            .filter_map(|(_, l)| match &l.content {
                LineContent::UnstagedFile(fc) => Some(fc.path.as_str()),
                _ => None,
            })
            .collect();
        if !files.is_empty() {
            return Selection::Files(files);
        }
    }

    // Check if all are staged files
    let all_staged_files = selected.iter().all(|(_, l)| {
        matches!(
            l.content,
            LineContent::StagedFile(_) | LineContent::SectionHeader { .. }
        )
    });
    if all_staged_files {
        let files: Vec<&str> = selected
            .iter()
            .filter_map(|(_, l)| match &l.content {
                LineContent::StagedFile(fc) => Some(fc.path.as_str()),
                _ => None,
            })
            .collect();
        if !files.is_empty() {
            return Selection::Files(files);
        }
    }

    // Check if all are diff lines in the same hunk (unstaged or staged)
    let all_diff_lines = selected
        .iter()
        .all(|(_, l)| matches!(l.content, LineContent::DiffLine(_)));
    if all_diff_lines {
        // Try unstaged first
        if let Some(selection) =
            try_get_lines_in_same_hunk(lines, selected, sel_start, sel_end, |s| {
                matches!(s, SectionType::UnstagedHunk { .. })
            })
        {
            return selection;
        }
        // Try staged
        if let Some(selection) =
            try_get_lines_in_same_hunk(lines, selected, sel_start, sel_end, |s| {
                matches!(s, SectionType::StagedHunk { .. })
            })
        {
            return selection;
        }
    }

    // Check if all are diff hunks (or diff lines within hunks) in the same file
    let all_hunks = selected.iter().all(|(_, l)| {
        matches!(
            l.content,
            LineContent::DiffHunk(_) | LineContent::DiffLine(_)
        )
    });
    if all_hunks {
        // Try unstaged first
        if let Some(selection) = try_get_hunks_in_same_file(selected, |s| match s {
            SectionType::UnstagedHunk { path, hunk_index } => Some((path.as_str(), *hunk_index)),
            _ => None,
        }) {
            return selection;
        }
        // Try staged
        if let Some(selection) = try_get_hunks_in_same_file(selected, |s| match s {
            SectionType::StagedHunk { path, hunk_index } => Some((path.as_str(), *hunk_index)),
            _ => None,
        }) {
            return selection;
        }
    }

    Selection::None
}

/// Try to get a Lines selection if all selected diff lines are in the same hunk.
fn try_get_lines_in_same_hunk<'a, F>(
    lines: &'a [Line],
    selected: &[(usize, &'a Line)],
    sel_start: usize,
    sel_end: usize,
    is_matching_hunk: F,
) -> Option<Selection<'a>>
where
    F: Fn(&SectionType) -> bool,
{
    let sections: Vec<&SectionType> = selected
        .iter()
        .filter_map(|(_, l)| l.section.as_ref())
        .collect();

    let first_section = sections.first()?;
    if !is_matching_hunk(first_section) {
        return None;
    }

    // Extract path and hunk_index from the first section
    let (path, hunk_index) = match first_section {
        SectionType::UnstagedHunk { path, hunk_index } => (path.as_str(), *hunk_index),
        SectionType::StagedHunk { path, hunk_index } => (path.as_str(), *hunk_index),
        _ => return None,
    };

    // Check all sections are the same hunk
    let all_same_hunk = sections.iter().all(|s| match s {
        SectionType::UnstagedHunk {
            path: p,
            hunk_index: h,
        } => p == path && *h == hunk_index,
        SectionType::StagedHunk {
            path: p,
            hunk_index: h,
        } => p == path && *h == hunk_index,
        _ => false,
    });

    if !all_same_hunk {
        return None;
    }

    // Calculate which lines within the hunk are selected.
    // Pass is_matching_hunk so only the correct section type (staged vs unstaged)
    // is counted, avoiding index inflation from the other section.
    let line_indices = compute_hunk_line_indices(
        lines,
        sel_start,
        sel_end,
        path,
        hunk_index,
        &is_matching_hunk,
    );
    if line_indices.is_empty() {
        return None;
    }

    Some(Selection::Lines {
        path,
        hunk_index,
        line_indices,
    })
}

/// Try to get a Hunks selection if all selected hunks are in the same file.
fn try_get_hunks_in_same_file<'a, F>(
    selected: &[(usize, &'a Line)],
    extract_hunk_info: F,
) -> Option<Selection<'a>>
where
    F: Fn(&SectionType) -> Option<(&str, usize)>,
{
    let hunk_info: Vec<(&str, usize)> = selected
        .iter()
        .filter_map(|(_, l)| l.section.as_ref().and_then(&extract_hunk_info))
        .collect();

    if hunk_info.is_empty() {
        return None;
    }

    let first_path = hunk_info[0].0;
    let all_same_file = hunk_info.iter().all(|(p, _)| *p == first_path);

    if !all_same_file {
        return None;
    }

    let mut indices: Vec<usize> = hunk_info.iter().map(|(_, i)| *i).collect();
    indices.sort_unstable();
    indices.dedup();
    // Reverse order for safe application (highest index first)
    indices.reverse();

    Some(Selection::Hunks {
        path: first_path,
        hunk_indices: indices,
    })
}

/// Computes the 0-based indices of diff lines within a hunk that are selected.
/// Counts only DiffLine entries (not the hunk header) in the matching section.
/// `is_matching_hunk` must be the same predicate used to identify the hunk type
/// (staged vs unstaged), so that lines from the other section type with the same
/// path and hunk_index are not mistakenly counted.
fn compute_hunk_line_indices<F>(
    lines: &[Line],
    sel_start: usize,
    sel_end: usize,
    target_path: &str,
    target_hunk_index: usize,
    is_matching_hunk: F,
) -> Vec<usize>
where
    F: Fn(&SectionType) -> bool,
{
    let mut result = Vec::new();
    let mut line_idx_in_hunk: usize = 0;

    for (i, line) in lines.iter().enumerate() {
        let is_matching_section = match &line.section {
            Some(section) if is_matching_hunk(section) => match section {
                SectionType::UnstagedHunk { path, hunk_index }
                | SectionType::StagedHunk { path, hunk_index } => {
                    path == target_path && *hunk_index == target_hunk_index
                }
                _ => false,
            },
            _ => false,
        };

        if !is_matching_section {
            continue;
        }

        if matches!(line.content, LineContent::DiffLine(_)) {
            if i >= sel_start && i <= sel_end {
                result.push(line_idx_in_hunk);
            }
            line_idx_in_hunk += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DiffHunk, FileChange, FileStatus};

    fn create_untracked_line(path: &str) -> Line {
        Line {
            content: LineContent::UntrackedFile(path.to_string()),
            section: Some(SectionType::UntrackedFiles),
        }
    }

    fn create_unstaged_file_line(path: &str) -> Line {
        Line {
            content: LineContent::UnstagedFile(FileChange {
                path: path.to_string(),
                status: FileStatus::Modified,
            }),
            section: Some(SectionType::UnstagedChanges),
        }
    }

    fn create_unstaged_hunk_line(path: &str, hunk_index: usize) -> Line {
        Line {
            content: LineContent::DiffHunk(DiffHunk {
                header: "@@ -1,3 +1,4 @@".to_string(),
                hunk_index,
            }),
            section: Some(SectionType::UnstagedHunk {
                path: path.to_string(),
                hunk_index,
            }),
        }
    }

    fn create_section_header(section: SectionType, title: &str) -> Line {
        Line {
            content: LineContent::SectionHeader {
                title: title.to_string(),
                count: Some(1),
            },
            section: Some(section),
        }
    }

    // Tests for Discardable context

    #[test]
    fn test_discardable_selection_returns_untracked_file() {
        let lines = vec![create_untracked_line("new_file.txt")];

        let selection = get_normal_mode_selection(&lines, 0, SelectionContext::Discardable);

        match selection {
            Selection::Files(files) => {
                assert_eq!(files.len(), 1);
                assert_eq!(files[0], "new_file.txt");
            }
            _ => panic!("Expected Selection::Files"),
        }
    }

    #[test]
    fn test_discardable_selection_returns_unstaged_file() {
        let lines = vec![create_unstaged_file_line("modified.txt")];

        let selection = get_normal_mode_selection(&lines, 0, SelectionContext::Discardable);

        match selection {
            Selection::Files(files) => {
                assert_eq!(files.len(), 1);
                assert_eq!(files[0], "modified.txt");
            }
            _ => panic!("Expected Selection::Files"),
        }
    }

    #[test]
    fn test_discardable_selection_returns_unstaged_hunk() {
        let lines = vec![create_unstaged_hunk_line("modified.txt", 0)];

        let selection = get_normal_mode_selection(&lines, 0, SelectionContext::Discardable);

        match selection {
            Selection::Hunk { path, hunk_index } => {
                assert_eq!(path, "modified.txt");
                assert_eq!(hunk_index, 0);
            }
            _ => panic!("Expected Selection::Hunk"),
        }
    }

    #[test]
    fn test_discardable_section_header_returns_all_unstaged_files() {
        let lines = vec![
            create_section_header(SectionType::UnstagedChanges, "Unstaged changes"),
            create_unstaged_file_line("file1.txt"),
            create_unstaged_file_line("file2.txt"),
        ];

        let selection = get_normal_mode_selection(&lines, 0, SelectionContext::Discardable);

        match selection {
            Selection::Files(files) => {
                assert_eq!(files.len(), 2);
                assert!(files.contains(&"file1.txt"));
                assert!(files.contains(&"file2.txt"));
            }
            _ => panic!("Expected Selection::Files"),
        }
    }

    #[test]
    fn test_discardable_untracked_section_header_returns_all_files() {
        let lines = vec![
            create_section_header(SectionType::UntrackedFiles, "Untracked files"),
            create_untracked_line("new_file1.txt"),
            create_untracked_line("new_file2.txt"),
        ];

        let selection = get_normal_mode_selection(&lines, 0, SelectionContext::Discardable);

        match selection {
            Selection::Files(files) => {
                assert_eq!(files.len(), 2);
                assert!(files.contains(&"new_file1.txt"));
                assert!(files.contains(&"new_file2.txt"));
            }
            _ => panic!("Expected Selection::Files"),
        }
    }

    #[test]
    fn test_discardable_visual_mode_multiple_files() {
        let lines = vec![
            create_unstaged_file_line("file1.txt"),
            create_unstaged_file_line("file2.txt"),
        ];

        let selection =
            get_visual_mode_selection(&lines, 0, 1, &HashSet::new(), SelectionContext::Discardable);

        match selection {
            Selection::Files(files) => {
                assert_eq!(files.len(), 2);
                assert!(files.contains(&"file1.txt"));
                assert!(files.contains(&"file2.txt"));
            }
            _ => panic!("Expected Selection::Files"),
        }
    }

    #[test]
    fn test_discardable_visual_mode_returns_untracked_files() {
        let lines = vec![
            create_untracked_line("new_file1.txt"),
            create_untracked_line("new_file2.txt"),
        ];

        let selection =
            get_visual_mode_selection(&lines, 0, 1, &HashSet::new(), SelectionContext::Discardable);

        match selection {
            Selection::Files(files) => {
                assert_eq!(files.len(), 2);
                assert!(files.contains(&"new_file1.txt"));
                assert!(files.contains(&"new_file2.txt"));
            }
            _ => panic!("Expected Selection::Files"),
        }
    }

    // Tests for staged discardable content

    fn create_staged_file_line(path: &str) -> Line {
        Line {
            content: LineContent::StagedFile(FileChange {
                path: path.to_string(),
                status: FileStatus::Modified,
            }),
            section: Some(SectionType::StagedChanges),
        }
    }

    fn create_staged_hunk_line(path: &str, hunk_index: usize) -> Line {
        Line {
            content: LineContent::DiffHunk(DiffHunk {
                header: "@@ -1,3 +1,4 @@".to_string(),
                hunk_index,
            }),
            section: Some(SectionType::StagedHunk {
                path: path.to_string(),
                hunk_index,
            }),
        }
    }

    #[test]
    fn test_discardable_selection_returns_staged_file() {
        let lines = vec![create_staged_file_line("staged.txt")];

        let selection = get_normal_mode_selection(&lines, 0, SelectionContext::Discardable);

        match selection {
            Selection::Files(files) => {
                assert_eq!(files.len(), 1);
                assert_eq!(files[0], "staged.txt");
            }
            _ => panic!("Expected Selection::Files"),
        }
    }

    #[test]
    fn test_discardable_selection_returns_staged_hunk() {
        let lines = vec![create_staged_hunk_line("staged.txt", 0)];

        let selection = get_normal_mode_selection(&lines, 0, SelectionContext::Discardable);

        match selection {
            Selection::Hunk { path, hunk_index } => {
                assert_eq!(path, "staged.txt");
                assert_eq!(hunk_index, 0);
            }
            _ => panic!("Expected Selection::Hunk"),
        }
    }

    #[test]
    fn test_discardable_section_header_returns_all_staged_files() {
        let lines = vec![
            create_section_header(SectionType::StagedChanges, "Staged changes"),
            create_staged_file_line("file1.txt"),
            create_staged_file_line("file2.txt"),
        ];

        let selection = get_normal_mode_selection(&lines, 0, SelectionContext::Discardable);

        match selection {
            Selection::Files(files) => {
                assert_eq!(files.len(), 2);
                assert!(files.contains(&"file1.txt"));
                assert!(files.contains(&"file2.txt"));
            }
            _ => panic!("Expected Selection::Files"),
        }
    }

    #[test]
    fn test_discardable_visual_mode_multiple_staged_files() {
        let lines = vec![
            create_staged_file_line("file1.txt"),
            create_staged_file_line("file2.txt"),
        ];

        let selection =
            get_visual_mode_selection(&lines, 0, 1, &HashSet::new(), SelectionContext::Discardable);

        match selection {
            Selection::Files(files) => {
                assert_eq!(files.len(), 2);
                assert!(files.contains(&"file1.txt"));
                assert!(files.contains(&"file2.txt"));
            }
            _ => panic!("Expected Selection::Files"),
        }
    }
}
