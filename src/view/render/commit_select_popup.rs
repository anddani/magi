use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::config::Theme;
use crate::model::popup::CommitSelectPopupState;
use crate::view::log_line;

/// Render the commit select popup as a bottom sheet (like command popups)
pub fn render(state: &CommitSelectPopupState, frame: &mut Frame, area: Rect, theme: &Theme) {
    // Calculate popup dimensions
    // Width: full width
    let popup_width = area.width;

    // Height: 50% of screen height (minimum 5 for header + border + at least 1 item)
    let popup_height = (area.height / 2).max(5);

    // Position: bottom of screen, full width
    let x = area.x;
    let y = area.y + area.height.saturating_sub(popup_height);
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    // Clear background
    frame.render_widget(Clear, popup_area);

    // Draw border with title
    let block = Block::default()
        .title(state.title.as_str())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.local_branch));
    frame.render_widget(block, popup_area);

    // Inner area (inside border)
    let inner = Rect::new(
        popup_area.x + 1,
        popup_area.y + 1,
        popup_area.width.saturating_sub(2),
        popup_area.height.saturating_sub(2),
    );

    // Header line: "[x]/[y] Title: input_"
    let display_index = if state.filtered_count() > 0 {
        state.selected_index + 1
    } else {
        0
    };
    let header_text = format!(
        "[{}]/[{}] {}: {}",
        display_index,
        state.filtered_count(),
        state.title,
        state.input_text
    );
    let header_line = Line::from(vec![
        Span::styled(header_text, Style::default()),
        Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
    ]);
    let header_area = Rect::new(inner.x, inner.y, inner.width, 1);
    frame.render_widget(Paragraph::new(header_line), header_area);

    // List area (below header)
    let list_area = Rect::new(
        inner.x,
        inner.y + 1,
        inner.width,
        inner.height.saturating_sub(1),
    );

    // Calculate scroll offset to keep selected item visible
    let visible_count = list_area.height as usize;
    let scroll_offset = if state.selected_index >= visible_count {
        state.selected_index - visible_count + 1
    } else {
        0
    };

    // Build list items with scrolling
    let items: Vec<ListItem> = state
        .filtered_indices
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_count)
        .map(|(i, &commit_idx)| {
            let commit = &state.all_commits[commit_idx];
            // Use log_line rendering for rich formatting
            let lines = log_line::get_lines(commit, theme, false, None);
            let is_selected = i == state.selected_index;

            // If selected, add background color to all spans in the line
            let styled_lines: Vec<Line> = if is_selected {
                lines
                    .into_iter()
                    .map(|mut line| {
                        for span in &mut line.spans {
                            span.style = span.style.bg(theme.selection_bg);
                        }
                        line
                    })
                    .collect()
            } else {
                lines
            };

            ListItem::new(styled_lines)
        })
        .collect();

    // Show "No matches" if list is empty
    if items.is_empty() {
        let no_matches = Paragraph::new(Span::styled(
            "No matches",
            Style::default().fg(ratatui::style::Color::DarkGray),
        ));
        frame.render_widget(no_matches, list_area);
    } else {
        let list = List::new(items);
        frame.render_widget(list, list_area);
    }
}
