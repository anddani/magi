//! Smart cursor positioning after UI changes.
//!
//! When items move or disappear (e.g., staging/unstaging files), the cursor
//! should move to a logical next position rather than disappearing or staying
//! on an invalid position.

use crate::model::{Line, LineContent, SectionType};

/// Context about what the cursor was on before a change.
/// Used to determine where to position the cursor after a refresh.
#[derive(Debug, Clone)]
pub enum CursorContext {
    /// Cursor was on an unstaged file
    UnstagedFile { path: String, line_index: usize },
    /// Cursor was on a staged file
    StagedFile { path: String, line_index: usize },
    /// Cursor was on an untracked file
    UntrackedFile { path: String, line_index: usize },
    /// Cursor was on a section header
    SectionHeader {
        section: SectionType,
        line_index: usize,
    },
    /// Cursor was on some other content
    Other { line_index: usize },
}

impl CursorContext {
    /// Captures the context at the current cursor position.
    pub fn capture(lines: &[Line], cursor_position: usize) -> Self {
        if cursor_position >= lines.len() {
            return CursorContext::Other {
                line_index: cursor_position,
            };
        }

        let line = &lines[cursor_position];

        match (&line.content, &line.section) {
            (LineContent::UnstagedFile(fc), _) => CursorContext::UnstagedFile {
                path: fc.path.clone(),
                line_index: cursor_position,
            },
            (LineContent::StagedFile(fc), _) => CursorContext::StagedFile {
                path: fc.path.clone(),
                line_index: cursor_position,
            },
            (LineContent::UntrackedFile(path), _) => CursorContext::UntrackedFile {
                path: path.clone(),
                line_index: cursor_position,
            },
            (LineContent::SectionHeader { .. }, Some(section)) => CursorContext::SectionHeader {
                section: section.clone(),
                line_index: cursor_position,
            },
            _ => CursorContext::Other {
                line_index: cursor_position,
            },
        }
    }

    /// Finds the best cursor position after a change.
    ///
    /// Strategy:
    /// 1. Try to find the same item (by path/section) if it still exists
    /// 2. Find the next item of the same type
    /// 3. Find the previous item of the same type
    /// 4. Find the section header for that type
    /// 5. Find the next section
    /// 6. Fallback to a safe position (clamped to bounds)
    pub fn find_best_position(&self, lines: &[Line]) -> usize {
        if lines.is_empty() {
            return 0;
        }

        match self {
            CursorContext::UnstagedFile { path, line_index } => find_next_or_prev_item(
                lines,
                *line_index,
                path,
                is_unstaged_file,
                Some(SectionType::UnstagedChanges),
            ),
            CursorContext::StagedFile { path, line_index } => find_next_or_prev_item(
                lines,
                *line_index,
                path,
                is_staged_file,
                Some(SectionType::StagedChanges),
            ),
            CursorContext::UntrackedFile { path, line_index } => find_next_or_prev_item(
                lines,
                *line_index,
                path,
                is_untracked_file,
                Some(SectionType::UntrackedFiles),
            ),
            CursorContext::SectionHeader {
                section,
                line_index,
            } => {
                // Try to stay on the same section if it exists
                if let Some(pos) = find_section_header(lines, section) {
                    return pos;
                }
                // Otherwise find next section after the original position
                find_next_section(lines, *line_index)
            }
            CursorContext::Other { line_index } => {
                // Just clamp to valid range
                (*line_index).min(lines.len().saturating_sub(1))
            }
        }
    }
}

/// Finds the next or previous item of a specific type.
///
/// Strategy:
/// 1. Check if the same item (by path) still exists - if so, return it
/// 2. Find the next item after original_index
/// 3. Find the previous item before original_index
/// 4. Find the previous section (section above - preferred when current section is empty)
/// 5. Find the section header for this type (if it still exists)
/// 6. Find the next section after original_index (fallback)
/// 7. Clamp to valid range
fn find_next_or_prev_item<F>(
    lines: &[Line],
    original_index: usize,
    original_path: &str,
    is_match: F,
    section_type: Option<SectionType>,
) -> usize
where
    F: Fn(&Line) -> Option<String>,
{
    // Try to find the same item (by path)
    if let Some(pos) = lines.iter().position(|line| {
        if let Some(path) = is_match(line) {
            path == original_path
        } else {
            false
        }
    }) {
        return pos;
    }

    // Find next item of the same type after original position
    for i in original_index..lines.len() {
        if is_match(&lines[i]).is_some() {
            return i;
        }
    }

    // Find previous item of the same type before original position
    for i in (0..original_index).rev() {
        if is_match(&lines[i]).is_some() {
            return i;
        }
    }

    // When the section becomes empty, try to find an adjacent section
    if let Some(section) = section_type.as_ref() {
        // First try to find the section header (if it still exists)
        if let Some(header_pos) = find_section_header(lines, section) {
            // If the section header exists, search for the previous section
            if let Some(pos) = find_prev_section(lines, header_pos) {
                return pos;
            }
            // If no previous section, return the section header itself
            return header_pos;
        }

        // Section no longer exists, find the best adjacent section
        if let Some(pos) = find_adjacent_section_for_type(lines, section) {
            return pos;
        }
    }

    // Fallback: find the previous section from original position
    if let Some(pos) = find_prev_section(lines, original_index) {
        return pos;
    }

    // Last resort: find next section after original position
    find_next_section(lines, original_index)
}

/// Finds the previous section header before the given position.
/// Returns None if no section is found before.
fn find_prev_section(lines: &[Line], from_index: usize) -> Option<usize> {
    // Search backward from the original position
    for i in (0..from_index).rev() {
        if matches!(
            lines[i].content,
            LineContent::SectionHeader { .. } | LineContent::UnpulledSectionHeader { .. }
        ) {
            return Some(i);
        }
    }
    None
}

/// Finds the next section header after the given position.
/// If no section is found after, searches before the position.
fn find_next_section(lines: &[Line], from_index: usize) -> usize {
    // First, search forward from the original position
    for i in from_index..lines.len() {
        if matches!(
            lines[i].content,
            LineContent::SectionHeader { .. } | LineContent::UnpulledSectionHeader { .. }
        ) {
            return i;
        }
    }

    // If no section found forward, search backward
    for i in (0..from_index).rev() {
        if matches!(
            lines[i].content,
            LineContent::SectionHeader { .. } | LineContent::UnpulledSectionHeader { .. }
        ) {
            return i;
        }
    }

    // No section found anywhere, clamp to valid range
    from_index.min(lines.len().saturating_sub(1))
}

/// Finds the section header for a specific section type.
fn find_section_header(lines: &[Line], section: &SectionType) -> Option<usize> {
    lines.iter().position(|line| {
        matches!(
            (&line.content, &line.section),
            (LineContent::SectionHeader { .. }, Some(s)) if s == section
        ) || matches!(
            (&line.content, &line.section),
            (LineContent::UnpulledSectionHeader { .. }, Some(s)) if s == section
        )
    })
}

fn is_unstaged_file(line: &Line) -> Option<String> {
    match &line.content {
        LineContent::UnstagedFile(fc) => Some(fc.path.clone()),
        _ => None,
    }
}

fn is_staged_file(line: &Line) -> Option<String> {
    match &line.content {
        LineContent::StagedFile(fc) => Some(fc.path.clone()),
        _ => None,
    }
}

fn is_untracked_file(line: &Line) -> Option<String> {
    match &line.content {
        LineContent::UntrackedFile(path) => Some(path.clone()),
        _ => None,
    }
}

/// Finds the best adjacent section when the current section is empty.
/// Tries to find sections in a logical order: previous section first, then next.
fn find_adjacent_section_for_type(lines: &[Line], section_type: &SectionType) -> Option<usize> {
    // Define the typical sections to look for when each section becomes empty
    let (prev_sections, next_sections): (Vec<SectionType>, Vec<SectionType>) = match section_type {
        SectionType::UntrackedFiles => (
            vec![SectionType::Info],
            vec![SectionType::UnstagedChanges, SectionType::StagedChanges],
        ),
        SectionType::UnstagedChanges => (
            vec![SectionType::UntrackedFiles, SectionType::Info],
            vec![SectionType::StagedChanges, SectionType::RecentCommits],
        ),
        SectionType::StagedChanges => (
            vec![
                SectionType::UnstagedChanges,
                SectionType::UntrackedFiles,
                SectionType::Info,
            ],
            vec![SectionType::RecentCommits, SectionType::Unpulled],
        ),
        _ => (vec![], vec![]),
    };

    // First, try to find a previous section
    for section in &prev_sections {
        if let Some(pos) = find_section_header(lines, section) {
            return Some(pos);
        }
    }

    // If no previous section found, try next sections
    for section in &next_sections {
        if let Some(pos) = find_section_header(lines, section) {
            return Some(pos);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FileChange, FileStatus};

    fn create_section_header(section: SectionType) -> Line {
        Line {
            content: LineContent::SectionHeader {
                title: "Test".to_string(),
                count: Some(1),
            },
            section: Some(section),
        }
    }

    fn create_unstaged_file(path: &str) -> Line {
        Line {
            content: LineContent::UnstagedFile(FileChange {
                path: path.to_string(),
                status: FileStatus::Modified,
            }),
            section: Some(SectionType::UnstagedFile {
                path: path.to_string(),
            }),
        }
    }

    fn create_staged_file(path: &str) -> Line {
        Line {
            content: LineContent::StagedFile(FileChange {
                path: path.to_string(),
                status: FileStatus::Modified,
            }),
            section: Some(SectionType::StagedFile {
                path: path.to_string(),
            }),
        }
    }

    #[test]
    fn test_cursor_moves_to_next_unstaged_file_when_current_is_staged() {
        let lines = vec![
            create_section_header(SectionType::UnstagedChanges),
            create_unstaged_file("file_b.txt"), // index 1
            create_unstaged_file("file_c.txt"), // index 2
        ];

        // Cursor was on file_a.txt which got staged (no longer exists)
        let context = CursorContext::UnstagedFile {
            path: "file_a.txt".to_string(),
            line_index: 1,
        };

        let new_pos = context.find_best_position(&lines);
        // Should move to file_b.txt (the next unstaged file)
        assert_eq!(new_pos, 1);
    }

    #[test]
    fn test_cursor_moves_to_section_header_when_all_files_staged() {
        let lines = vec![
            create_section_header(SectionType::UnstagedChanges),
            create_section_header(SectionType::StagedChanges),
        ];

        // Cursor was on file_a.txt which was the last unstaged file
        let context = CursorContext::UnstagedFile {
            path: "file_a.txt".to_string(),
            line_index: 1,
        };

        let new_pos = context.find_best_position(&lines);
        // Should move to unstaged section header or next section
        assert!(new_pos <= 1);
    }

    #[test]
    fn test_cursor_stays_on_same_file_if_it_still_exists() {
        let lines = vec![
            create_section_header(SectionType::UnstagedChanges),
            create_unstaged_file("file_a.txt"),
            create_unstaged_file("file_b.txt"),
        ];

        let context = CursorContext::UnstagedFile {
            path: "file_a.txt".to_string(),
            line_index: 1,
        };

        let new_pos = context.find_best_position(&lines);
        // Should stay on file_a.txt
        assert_eq!(new_pos, 1);
    }

    #[test]
    fn test_cursor_moves_to_next_section_when_current_section_disappears() {
        let lines = vec![
            create_section_header(SectionType::StagedChanges),
            create_staged_file("staged.txt"),
        ];

        // Unstaged section disappeared
        let context = CursorContext::SectionHeader {
            section: SectionType::UnstagedChanges,
            line_index: 1,
        };

        let new_pos = context.find_best_position(&lines);
        // Should move to the staged changes section (next section)
        assert_eq!(new_pos, 0);
    }

    #[test]
    fn test_cursor_moves_to_previous_file_when_last_file_staged() {
        let lines = vec![
            create_section_header(SectionType::UnstagedChanges),
            create_unstaged_file("file_a.txt"),
        ];

        // file_b.txt (which was after file_a) was staged
        let context = CursorContext::UnstagedFile {
            path: "file_b.txt".to_string(),
            line_index: 2, // was at index 2
        };

        let new_pos = context.find_best_position(&lines);
        // Should move to file_a.txt (previous file) since there's no next file
        assert_eq!(new_pos, 1);
    }
}
