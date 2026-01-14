use ratatui::{
    text::{Line as TextLine, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    model::Model,
    view::{
        render::{render_dialog, render_toast},
        util::{selection_style, visible_scroll_offset},
    },
};

mod util;

mod diff_hunk;
mod diff_line;
mod head_ref;
mod latest_tag;
mod push_ref;
mod render;
mod section_header;
mod staged_file;
mod unstaged_file;
mod untracked_file;

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
        let is_in_selected_section = util::should_highlight_line(
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
                    util::apply_block_cursor(text_line, 1);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        DiffHunk, DiffLine, DiffLineType, FileChange, Line, LineContent, SectionType,
    };
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
