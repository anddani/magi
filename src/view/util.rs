use std::collections::HashSet;

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line as TextLine, Span},
};

use crate::git::{GitRef, ReferenceType};
use crate::model::{FileChange, FileStatus, LineContent, SectionType};
use crate::{config::Theme, model::Line};

/// Style for the highlighted section (faded background)
pub fn selection_style(bg_color: Color) -> Style {
    Style::default().bg(bg_color)
}

/// Converts a raw line index scroll offset to visible line count.
/// The model stores scroll_offset as a raw index into the lines array,
/// but Paragraph::scroll expects the number of rendered (visible) lines to skip.
pub fn visible_scroll_offset(
    lines: &[Line],
    scroll_offset: usize,
    collapsed_sections: &HashSet<SectionType>,
) -> usize {
    lines
        .iter()
        .take(scroll_offset)
        .filter(|line| !line.is_hidden(collapsed_sections))
        .count()
}

/// Format a GitRef with appropriate colors for different parts using Ratatui's styling system
pub fn format_ref_with_colors<'a>(ref_info: &GitRef, label: &str, theme: &Theme) -> Vec<Span<'a>> {
    let mut spans = vec![Span::styled(
        label.to_string(),
        Style::default().fg(theme.ref_label),
    )];

    // Branch name with appropriate color
    let branch_style = match ref_info.reference_type {
        ReferenceType::RemoteBranch => Style::default().fg(theme.remote_branch),
        ReferenceType::DetachedHead => Style::default().fg(theme.detached_head),
        ReferenceType::LocalBranch => Style::default().fg(theme.local_branch),
    };

    spans.push(Span::styled(ref_info.name.clone(), branch_style));
    spans.push(Span::styled(" ", Style::default()));

    // Git hash
    spans.push(Span::styled(
        ref_info.commit_hash.clone(),
        Style::default().fg(theme.commit_hash),
    ));

    spans.push(Span::styled(" ", Style::default()));

    // Commit message
    spans.push(Span::styled(
        ref_info.commit_message.clone(),
        Style::default().fg(theme.text),
    ));

    spans
}

/// Generate the view line for a file change (staged or unstaged)
pub fn format_file_change(
    file_change: &FileChange,
    collapsed: bool,
    status_color: Color,
    theme: &Theme,
) -> TextLine<'static> {
    let status_str = match file_change.status {
        FileStatus::Modified => "modified",
        FileStatus::Deleted => "deleted",
        FileStatus::New => "new file",
        FileStatus::Renamed => "renamed",
        FileStatus::Copied => "copied",
        FileStatus::TypeChange => "typechange",
    };

    // Use '>' when collapsed, '∨' when expanded
    let indicator = if collapsed { ">" } else { "∨" };

    TextLine::from(vec![
        Span::raw(indicator),
        Span::styled(
            format!("{} ", status_str),
            Style::default().fg(status_color),
        ),
        Span::styled(
            file_change.path.clone(),
            Style::default().fg(theme.file_path),
        ),
    ])
}

/// Style for the block cursor (reversed colors like vim)
fn block_cursor_style() -> Style {
    Style::default().add_modifier(Modifier::REVERSED)
}

/// Apply a block cursor style to a specific column within a text line.
/// This splits spans as needed to style only that single character.
/// Uses character (not byte) indexing to handle UTF-8 properly.
pub fn apply_block_cursor(text_line: &mut TextLine, column: usize) {
    // Clone spans to owned strings to avoid lifetime issues
    let original_spans: Vec<(String, Style)> = text_line
        .spans
        .iter()
        .map(|s| (s.content.to_string(), s.style))
        .collect();

    let mut new_spans: Vec<Span> = Vec::new();
    let mut current_col = 0;
    let mut cursor_applied = false;

    for (content, style) in original_spans {
        let char_count = content.chars().count();
        let span_start = current_col;
        let span_end = current_col + char_count;

        if !cursor_applied && column >= span_start && column < span_end {
            // The cursor falls within this span - split it
            let char_offset = column - span_start;
            let chars: Vec<char> = content.chars().collect();

            // Part before cursor
            if char_offset > 0 {
                let before: String = chars[..char_offset].iter().collect();
                new_spans.push(Span::styled(before, style));
            }

            // The cursor character (single char with reversed style)
            if char_offset < chars.len() {
                let cursor_char = chars[char_offset].to_string();
                new_spans.push(Span::styled(cursor_char, style.patch(block_cursor_style())));
            }

            // Part after cursor
            if char_offset + 1 < chars.len() {
                let after: String = chars[char_offset + 1..].iter().collect();
                new_spans.push(Span::styled(after, style));
            }

            cursor_applied = true;
        } else {
            new_spans.push(Span::styled(content, style));
        }

        current_col = span_end;
    }

    // If the line is shorter than the cursor column, we need to pad and show cursor
    if !cursor_applied {
        let padding = column.saturating_sub(current_col);
        if padding > 0 {
            new_spans.push(Span::raw(" ".repeat(padding)));
        }
        new_spans.push(Span::styled(" ", block_cursor_style()));
    }

    text_line.spans = new_spans;
}

/// Apply selection highlighting and optional block cursor to text lines.
/// Pads lines to fill the content width with the selection background.
pub fn apply_selection_style(
    line_texts: &mut Vec<TextLine>,
    content_width: usize,
    is_cursor_line: bool,
    selection_bg: Color,
) {
    let sel_style = selection_style(selection_bg);
    for text_line in line_texts {
        let line_width: usize = text_line.spans.iter().map(|s| s.content.len()).sum();
        let padding = content_width.saturating_sub(line_width);
        let mut spans: Vec<Span> = text_line.spans.clone();
        if padding > 0 {
            spans.push(Span::styled(" ".repeat(padding), sel_style));
        }
        *text_line = TextLine::from(spans).style(sel_style);

        if is_cursor_line {
            apply_block_cursor(text_line, 1);
        }
    }
}

/// Determines whether a line should be highlighted based on cursor position and section context.
///
/// # Arguments
/// - `line_index`: the index of the line being checked
/// - `cursor_index`: the index of the cursor line
/// - `cursor_content`: the content of the cursor line
/// - `cursor_section`: the section of the line at cursor position
/// - `line_section`: the section of the line being checked
pub fn should_highlight_line(
    line_index: usize,
    cursor_index: usize,
    cursor_content: Option<&LineContent>,
    cursor_section: Option<&SectionType>,
    line_section: Option<&SectionType>,
) -> bool {
    if line_index == cursor_index {
        return true;
    }

    match (cursor_content, cursor_section, line_section) {
        // Cursor on SectionHeader with UnstagedChanges: highlight all unstaged lines
        (
            Some(LineContent::SectionHeader { .. }),
            Some(SectionType::UnstagedChanges),
            Some(line_sec),
        ) => {
            matches!(
                line_sec,
                SectionType::UnstagedChanges
                    | SectionType::UnstagedFile { .. }
                    | SectionType::UnstagedHunk { .. }
            )
        }
        // Cursor on SectionHeader with StagedChanges: highlight all staged lines
        (
            Some(LineContent::SectionHeader { .. }),
            Some(SectionType::StagedChanges),
            Some(line_sec),
        ) => {
            matches!(
                line_sec,
                SectionType::StagedChanges
                    | SectionType::StagedFile { .. }
                    | SectionType::StagedHunk { .. }
            )
        }
        // Cursor on other SectionHeader: highlight all lines with same section
        (Some(LineContent::SectionHeader { .. }), Some(cursor_sec), Some(line_sec)) => {
            cursor_sec == line_sec
        }
        // Cursor on HeadRef: highlight all lines in Info section
        (Some(LineContent::HeadRef(_)), _, Some(SectionType::Info)) => true,
        // Cursor on UnstagedFile: highlight file line + all its hunks
        (
            Some(LineContent::UnstagedFile(_)),
            Some(SectionType::UnstagedFile { path: cursor_path }),
            Some(line_sec),
        ) => match line_sec {
            SectionType::UnstagedFile { path } => path == cursor_path,
            SectionType::UnstagedHunk { path, .. } => path == cursor_path,
            _ => false,
        },
        // Cursor on StagedFile: highlight file line + all its hunks
        (
            Some(LineContent::StagedFile(_)),
            Some(SectionType::StagedFile { path: cursor_path }),
            Some(line_sec),
        ) => match line_sec {
            SectionType::StagedFile { path } => path == cursor_path,
            SectionType::StagedHunk { path, .. } => path == cursor_path,
            _ => false,
        },
        // Cursor on DiffHunk header: highlight the hunk section
        (Some(LineContent::DiffHunk(_)), Some(cursor_sec), Some(line_sec)) => {
            cursor_sec == line_sec
        }
        // Otherwise, don't highlight other lines
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::{GitRef, ReferenceType};
    use crate::model::{DiffHunk, DiffLine, DiffLineType};

    fn create_test_lines_with_sections() -> Vec<Line> {
        use crate::model::FileStatus;

        vec![
            // 0: Section header (visible)
            Line {
                content: LineContent::SectionHeader {
                    title: "Unstaged changes".to_string(),
                    count: Some(1),
                },
                section: Some(SectionType::UnstagedChanges),
            },
            // 1: File header (visible, acts as header for its section)
            Line {
                content: LineContent::UnstagedFile(FileChange {
                    path: "foo.rs".to_string(),
                    status: FileStatus::Modified,
                }),
                section: Some(SectionType::UnstagedFile {
                    path: "foo.rs".to_string(),
                }),
            },
            // 2: Hunk (can be hidden when file is collapsed)
            Line {
                content: LineContent::DiffHunk(DiffHunk {
                    header: "@@ -1,5 +1,6 @@".to_string(),
                }),
                section: Some(SectionType::UnstagedHunk {
                    path: "foo.rs".to_string(),
                    hunk_index: 0,
                }),
            },
            // 3: Diff line (can be hidden)
            Line {
                content: LineContent::DiffLine(DiffLine {
                    content: "+ added".to_string(),
                    line_type: DiffLineType::Addition,
                }),
                section: Some(SectionType::UnstagedHunk {
                    path: "foo.rs".to_string(),
                    hunk_index: 0,
                }),
            },
            // 4: Diff line (can be hidden)
            Line {
                content: LineContent::DiffLine(DiffLine {
                    content: "- removed".to_string(),
                    line_type: DiffLineType::Deletion,
                }),
                section: Some(SectionType::UnstagedHunk {
                    path: "foo.rs".to_string(),
                    hunk_index: 0,
                }),
            },
            // 5: Empty line (always visible)
            Line {
                content: LineContent::EmptyLine,
                section: None,
            },
            // 6: Another section header
            Line {
                content: LineContent::SectionHeader {
                    title: "Untracked files".to_string(),
                    count: Some(1),
                },
                section: Some(SectionType::UntrackedFiles),
            },
        ]
    }

    #[test]
    fn test_cursor_line_always_highlighted() {
        // The cursor line itself should always be highlighted
        assert!(should_highlight_line(5, 5, None, None, None));
    }

    #[test]
    fn test_unstaged_changes_header_highlights_all_unstaged_lines() {
        let header_content = LineContent::SectionHeader {
            title: "Unstaged changes".to_string(),
            count: Some(2),
        };
        let cursor_section = SectionType::UnstagedChanges;

        // Header should highlight itself
        assert!(should_highlight_line(
            1,
            0,
            Some(&header_content),
            Some(&cursor_section),
            Some(&SectionType::UnstagedChanges),
        ));

        // Header should highlight file lines
        assert!(should_highlight_line(
            1,
            0,
            Some(&header_content),
            Some(&cursor_section),
            Some(&SectionType::UnstagedFile {
                path: "src/main.rs".to_string()
            }),
        ));

        // Header should highlight hunk lines
        assert!(should_highlight_line(
            2,
            0,
            Some(&header_content),
            Some(&cursor_section),
            Some(&SectionType::UnstagedHunk {
                path: "src/main.rs".to_string(),
                hunk_index: 0
            }),
        ));

        // Header should NOT highlight unrelated sections
        assert!(!should_highlight_line(
            3,
            0,
            Some(&header_content),
            Some(&cursor_section),
            Some(&SectionType::UntrackedFiles),
        ));
    }

    #[test]
    fn test_head_ref_highlights_all_info_lines() {
        let head_ref_content = LineContent::HeadRef(GitRef::new(
            "main".to_string(),
            "abc1234".to_string(),
            "Initial commit".to_string(),
            ReferenceType::LocalBranch,
        ));
        let cursor_section = SectionType::Info;

        // HeadRef should highlight all Info section lines
        assert!(should_highlight_line(
            1,
            0,
            Some(&head_ref_content),
            Some(&cursor_section),
            Some(&SectionType::Info),
        ));

        // HeadRef should NOT highlight other sections
        assert!(!should_highlight_line(
            2,
            0,
            Some(&head_ref_content),
            Some(&cursor_section),
            Some(&SectionType::UntrackedFiles),
        ));
    }

    #[test]
    fn test_unstaged_file_highlights_file_and_its_hunks() {
        let file_content = LineContent::UnstagedFile(FileChange {
            path: "src/main.rs".to_string(),
            status: FileStatus::Modified,
        });
        let cursor_section = SectionType::UnstagedFile {
            path: "src/main.rs".to_string(),
        };

        // File should highlight its own hunks
        assert!(should_highlight_line(
            2,
            1,
            Some(&file_content),
            Some(&cursor_section),
            Some(&SectionType::UnstagedHunk {
                path: "src/main.rs".to_string(),
                hunk_index: 0
            }),
        ));

        // File should NOT highlight hunks from other files
        assert!(!should_highlight_line(
            5,
            1,
            Some(&file_content),
            Some(&cursor_section),
            Some(&SectionType::UnstagedHunk {
                path: "src/other.rs".to_string(),
                hunk_index: 0
            }),
        ));
    }

    #[test]
    fn test_staged_changes_header_highlights_all_staged_lines() {
        let header_content = LineContent::SectionHeader {
            title: "Staged changes".to_string(),
            count: Some(2),
        };
        let cursor_section = SectionType::StagedChanges;

        // Header should highlight itself
        assert!(should_highlight_line(
            1,
            0,
            Some(&header_content),
            Some(&cursor_section),
            Some(&SectionType::StagedChanges),
        ));

        // Header should highlight file lines
        assert!(should_highlight_line(
            1,
            0,
            Some(&header_content),
            Some(&cursor_section),
            Some(&SectionType::StagedFile {
                path: "src/main.rs".to_string()
            }),
        ));

        // Header should highlight hunk lines
        assert!(should_highlight_line(
            2,
            0,
            Some(&header_content),
            Some(&cursor_section),
            Some(&SectionType::StagedHunk {
                path: "src/main.rs".to_string(),
                hunk_index: 0
            }),
        ));

        // Header should NOT highlight unrelated sections
        assert!(!should_highlight_line(
            3,
            0,
            Some(&header_content),
            Some(&cursor_section),
            Some(&SectionType::UntrackedFiles),
        ));
    }

    #[test]
    fn test_staged_file_highlights_file_and_its_hunks() {
        let file_content = LineContent::StagedFile(FileChange {
            path: "src/main.rs".to_string(),
            status: FileStatus::Modified,
        });
        let cursor_section = SectionType::StagedFile {
            path: "src/main.rs".to_string(),
        };

        // File should highlight its own hunks
        assert!(should_highlight_line(
            2,
            1,
            Some(&file_content),
            Some(&cursor_section),
            Some(&SectionType::StagedHunk {
                path: "src/main.rs".to_string(),
                hunk_index: 0
            }),
        ));

        // File should NOT highlight hunks from other files
        assert!(!should_highlight_line(
            5,
            1,
            Some(&file_content),
            Some(&cursor_section),
            Some(&SectionType::StagedHunk {
                path: "src/other.rs".to_string(),
                hunk_index: 0
            }),
        ));
    }

    #[test]
    fn test_diff_hunk_highlights_only_its_section() {
        let hunk_content = LineContent::DiffHunk(DiffHunk {
            header: "@@ -1,5 +1,6 @@".to_string(),
        });
        let cursor_section = SectionType::UnstagedHunk {
            path: "src/main.rs".to_string(),
            hunk_index: 0,
        };

        // Hunk should highlight lines in the same hunk
        assert!(should_highlight_line(
            3,
            2,
            Some(&hunk_content),
            Some(&cursor_section),
            Some(&SectionType::UnstagedHunk {
                path: "src/main.rs".to_string(),
                hunk_index: 0
            }),
        ));

        // Hunk should NOT highlight lines in different hunks
        assert!(!should_highlight_line(
            5,
            2,
            Some(&hunk_content),
            Some(&cursor_section),
            Some(&SectionType::UnstagedHunk {
                path: "src/main.rs".to_string(),
                hunk_index: 1
            }),
        ));
    }

    #[test]
    fn test_diff_line_highlights_only_itself() {
        let diff_line_content = LineContent::DiffLine(DiffLine {
            content: "+ new line".to_string(),
            line_type: DiffLineType::Addition,
        });
        let cursor_section = SectionType::UnstagedHunk {
            path: "src/main.rs".to_string(),
            hunk_index: 0,
        };

        // DiffLine should NOT highlight other lines (only cursor line is highlighted)
        assert!(!should_highlight_line(
            4,
            3,
            Some(&diff_line_content),
            Some(&cursor_section),
            Some(&SectionType::UnstagedHunk {
                path: "src/main.rs".to_string(),
                hunk_index: 0
            }),
        ));
    }

    #[test]
    fn test_visible_scroll_offset_no_hidden_lines() {
        let lines = create_test_lines_with_sections();
        let collapsed = HashSet::new();

        // With no collapsed sections, visible offset equals raw offset
        assert_eq!(visible_scroll_offset(&lines, 0, &collapsed), 0);
        assert_eq!(visible_scroll_offset(&lines, 3, &collapsed), 3);
        assert_eq!(visible_scroll_offset(&lines, 5, &collapsed), 5);
    }

    #[test]
    fn test_visible_scroll_offset_with_collapsed_file() {
        let lines = create_test_lines_with_sections();
        let mut collapsed = HashSet::new();
        // Collapse the file section - this hides hunks (lines 2, 3, 4)
        collapsed.insert(SectionType::UnstagedFile {
            path: "foo.rs".to_string(),
        });

        // Lines 0, 1 are visible; lines 2, 3, 4 are hidden; lines 5, 6 are visible
        // scroll_offset=0 -> 0 visible lines before
        assert_eq!(visible_scroll_offset(&lines, 0, &collapsed), 0);
        // scroll_offset=2 -> lines 0, 1 are visible = 2
        assert_eq!(visible_scroll_offset(&lines, 2, &collapsed), 2);
        // scroll_offset=5 -> lines 0, 1 visible, lines 2, 3, 4 hidden = 2
        assert_eq!(visible_scroll_offset(&lines, 5, &collapsed), 2);
        // scroll_offset=6 -> lines 0, 1, 5 visible = 3
        assert_eq!(visible_scroll_offset(&lines, 6, &collapsed), 3);
        // scroll_offset=7 (all lines) -> lines 0, 1, 5, 6 visible = 4
        assert_eq!(visible_scroll_offset(&lines, 7, &collapsed), 4);
    }

    #[test]
    fn test_visible_scroll_offset_with_collapsed_main_section() {
        let lines = create_test_lines_with_sections();
        let mut collapsed = HashSet::new();
        // Collapse the main "Unstaged changes" section - this hides lines 1-4
        collapsed.insert(SectionType::UnstagedChanges);

        // Lines 0 (header), 5 (empty), 6 (header) are visible
        // scroll_offset=5 -> only line 0 is visible before = 1
        assert_eq!(visible_scroll_offset(&lines, 5, &collapsed), 1);
        // scroll_offset=6 -> lines 0, 5 visible = 2
        assert_eq!(visible_scroll_offset(&lines, 6, &collapsed), 2);
    }

    #[test]
    fn test_visible_scroll_offset_beyond_line_count() {
        let lines = create_test_lines_with_sections();
        let collapsed = HashSet::new();

        // scroll_offset beyond line count should count all visible lines
        assert_eq!(visible_scroll_offset(&lines, 100, &collapsed), 7);
    }
}
