use ratatui::{
    text::Line as TextLine,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    model::Model,
    view::{
        render::{render_dialog, render_toast},
        util::{apply_selection_style, visible_scroll_offset},
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

    // Get visual selection range if visual mode is active
    let visual_range = model.ui_model.visual_selection_range();

    // Determine what should be highlighted based on cursor position (for normal mode)
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
        // In visual mode, highlight all visible lines between anchor and cursor
        let is_in_selected_section = if let Some((start, end)) = visual_range {
            index >= start && index <= end
        } else {
            util::should_highlight_line(
                index,
                cursor_pos,
                cursor_content,
                cursor_section,
                line.section.as_ref(),
            )
        };

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

        if is_in_selected_section {
            apply_selection_style(
                &mut line_texts,
                content_width,
                is_cursor_line,
                theme.selection_bg,
            );
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
