use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line as TextLine, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use std::collections::HashSet;

use crate::model::{DialogContent, Line, LineContent, Model, SectionType, Toast, ToastStyle};

mod util;

mod diff_hunk;
mod diff_line;
mod head_ref;
mod latest_tag;
mod push_ref;
mod section_header;
mod staged_file;
mod unstaged_file;
mod untracked_file;

/// Style for the highlighted section (faded background)
fn selection_style(bg_color: Color) -> Style {
    Style::default().bg(bg_color)
}

/// Style for the block cursor (reversed colors like vim)
fn block_cursor_style() -> Style {
    Style::default().add_modifier(Modifier::REVERSED)
}

/// Converts a raw line index scroll offset to visible line count.
/// The model stores scroll_offset as a raw index into the lines array,
/// but Paragraph::scroll expects the number of rendered (visible) lines to skip.
fn visible_scroll_offset(
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

/// Determines if a line should be highlighted based on the cursor position.
/// Returns true if the line at `line_index` should be highlighted given:
/// - `cursor_index`: the current cursor position
/// - `cursor_content`: the content of the line at cursor position
/// - `cursor_section`: the section of the line at cursor position
/// - `line_section`: the section of the line being checked
fn should_highlight_line(
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

/// Apply a block cursor at the specified column position in the line.
/// This splits spans as needed to style only that single character.
/// Uses character (not byte) indexing to handle UTF-8 properly.
fn apply_block_cursor(text_line: &mut TextLine, column: usize) {
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

/// The view functions draws the UI using the application
/// state (Model).
///
/// ┌─────────────────────────────────────────────────┐
/// |∨Head:     origin/main Initial commit            |
/// | Push:     origin/main Initial commit            |
/// | Tag:      v17 (42)                              |
/// |                                                 |
/// |∨Untracked files (1)                             |
/// | src/foo/bar.rs                                  |
/// |                                                 |
/// |∨Unstaged changes (1)                            |
/// | modified src/main.rs                            |
/// | @@ -7,6 +7,7 @@use std::time::Duration;         |
/// |                                                 |
/// |  use crate::{                                   |
/// | + errors::MagiResult,                           |
/// |                                                 |
/// |∨Staged changes (1)                              |
/// | modified src/main.rs                            |
/// | @@ -20,7 +20,7 @@use ratatui::Frame;            |
/// | mod view;                                       |
/// |  fn main() -> MagiResult<()> {                  |
/// | -    magi::run()?                               |
/// | +    magi::run()?;                              |
/// |      Ok(())                                     |
/// |  }                                              |
/// |                                                 |
/// |∨Recent commits                                  |
/// | 8002f05 origin/main Add Elm architecture        |
/// | 467e2a7 Add ratatui hello world                 |
/// | bce473e Initial commit                          |
/// |                                                 |
/// └─────────────────────────────────────────────────┘
///
///
pub fn view(model: &Model, frame: &mut Frame) {
    let area = frame.area();
    let mut text = Vec::new();
    let theme = &model.theme;
    let cursor_pos = model.ui_model.cursor_position;
    // Content width is area width minus 2 for borders
    let content_width = area.width.saturating_sub(2) as usize;

    // Determine what should be highlighted based on cursor position
    let cursor_line = model.ui_model.lines.get(cursor_pos);
    let cursor_content = cursor_line.map(|l| &l.content);
    let cursor_section = cursor_line.and_then(|l| l.section.as_ref());

    let collapsed_sections = &model.ui_model.collapsed_sections;

    for (index, line) in model.ui_model.lines.iter().enumerate() {
        // Skip hidden lines (lines whose parent section is collapsed)
        if line.is_hidden(collapsed_sections) {
            continue;
        }

        // Determine if this line should be highlighted
        let is_in_selected_section = should_highlight_line(
            index,
            cursor_pos,
            cursor_content,
            cursor_section,
            line.section.as_ref(),
        );

        // Check if this line's section is collapsed (for showing the indicator)
        let is_section_collapsed = if let Some(section) = line.collapsible_section() {
            collapsed_sections.contains(&section)
        } else {
            false
        };

        let mut line_texts: Vec<TextLine> = match &line.content {
            crate::model::LineContent::EmptyLine => vec![TextLine::from("")],
            crate::model::LineContent::HeadRef(git_ref) => {
                head_ref::get_lines(git_ref, is_section_collapsed, theme)
            }
            crate::model::LineContent::PushRef(git_ref) => push_ref::get_lines(git_ref, theme),
            crate::model::LineContent::Tag(tag_info) => latest_tag::get_lines(tag_info, theme),
            crate::model::LineContent::SectionHeader { title, count } => {
                section_header::get_lines(title, *count, is_section_collapsed, theme)
            }
            crate::model::LineContent::UntrackedFile(file_path) => {
                untracked_file::get_lines(file_path, theme)
            }
            crate::model::LineContent::UnstagedFile(file_change) => {
                unstaged_file::get_lines(file_change, is_section_collapsed, theme)
            }
            crate::model::LineContent::StagedFile(file_change) => {
                staged_file::get_lines(file_change, is_section_collapsed, theme)
            }
            crate::model::LineContent::DiffHunk(hunk) => diff_hunk::get_lines(hunk, theme),
            crate::model::LineContent::DiffLine(diff_line) => {
                diff_line::get_lines(diff_line, theme)
            }
        };

        let is_cursor_line = index == cursor_pos;

        // Apply cursor highlighting to all lines in the selected section
        if is_in_selected_section {
            let sel_style = selection_style(theme.selection_bg);
            for text_line in &mut line_texts {
                // Calculate current line width and add padding to fill the screen
                let line_width: usize = text_line.spans.iter().map(|s| s.content.len()).sum();
                let padding = content_width.saturating_sub(line_width);
                let mut spans: Vec<Span> = text_line.spans.clone();
                if padding > 0 {
                    spans.push(Span::styled(" ".repeat(padding), sel_style));
                }
                *text_line = TextLine::from(spans).style(sel_style);

                // Apply block cursor only on the actual cursor line, at column 1 (second character)
                if is_cursor_line {
                    apply_block_cursor(text_line, 1);
                }
            }
        }

        text.extend(line_texts);
    }

    let scroll = visible_scroll_offset(
        &model.ui_model.lines,
        model.ui_model.scroll_offset,
        collapsed_sections,
    ) as u16;
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Magi"))
        .scroll((scroll, 0));

    frame.render_widget(paragraph, area);

    // Render toast in bottom-right corner if present
    if let Some(toast) = &model.toast {
        render_toast(toast, frame, area, theme);
    }

    // Render dialog overlay if present (on top of toast)
    if let Some(dialog) = &model.dialog {
        render_dialog(dialog, frame, area, theme);
    }
}

/// Calculate a centered rectangle within the given area
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

/// Calculate a rectangle in the bottom-right corner
fn bottom_right_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width + 1);
    let y = area.y + area.height.saturating_sub(height + 1);
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

/// Render a toast notification in the bottom-right corner
fn render_toast(toast: &Toast, frame: &mut Frame, area: Rect, theme: &crate::config::Theme) {
    let border_color = match toast.style {
        ToastStyle::Success => theme.staged_status,
        ToastStyle::Info => theme.local_branch,
        ToastStyle::Warning => theme.section_header,
    };

    // Calculate toast size based on content
    let content_width = toast.message.len() + 4; // padding
    let toast_width = (content_width as u16).clamp(20, area.width.saturating_sub(4));
    let toast_height = 3; // border + content + border

    let toast_area = bottom_right_rect(toast_width, toast_height, area);

    // Clear the area behind the toast
    frame.render_widget(Clear, toast_area);

    let toast_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let toast_paragraph = Paragraph::new(toast.message.as_str()).block(toast_block);

    frame.render_widget(toast_paragraph, toast_area);
}

/// Render a modal dialog overlay (centered, requires user action)
fn render_dialog(
    dialog: &DialogContent,
    frame: &mut Frame,
    area: Rect,
    theme: &crate::config::Theme,
) {
    let (title, content, border_color) = match dialog {
        DialogContent::Error { message } => ("Error", message.as_str(), theme.diff_deletion),
    };

    // Calculate dialog size based on content
    let content_width = content.len().max(title.len()) + 4; // padding
    let dialog_width = (content_width as u16).clamp(30, area.width.saturating_sub(4));
    let dialog_height = 5; // title bar + content + border + hint

    let dialog_area = centered_rect(dialog_width, dialog_height, area);

    // Clear the area behind the dialog
    frame.render_widget(Clear, dialog_area);

    // Build dialog content with hint
    let hint = "Press Enter or Esc to dismiss";
    let dialog_text = vec![
        TextLine::from(content),
        TextLine::from(""),
        TextLine::from(Span::styled(hint, Style::default().fg(Color::DarkGray))),
    ];

    let dialog_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let dialog_paragraph = Paragraph::new(dialog_text).block(dialog_block);

    frame.render_widget(dialog_paragraph, dialog_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DiffHunk, DiffLine, DiffLineType, FileChange, FileStatus};

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
        use crate::git::{GitRef, ReferenceType};

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

    use crate::model::Line;
    use std::collections::HashSet;

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
