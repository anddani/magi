use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::config::Theme;
use crate::model::popup::SelectPopupState;

/// Render the select popup (fuzzy finder style)
pub fn render(state: &SelectPopupState, frame: &mut Frame, area: Rect, theme: &Theme) {
    // Calculate popup dimensions
    // Width: 60% of screen, minimum 40, maximum screen width - 4
    let popup_width = (area.width * 60 / 100)
        .max(40)
        .min(area.width.saturating_sub(4));

    // Height: header (1) + list (up to half screen) + border (2)
    let max_list_height = (area.height / 2).saturating_sub(3);
    let list_height = (state.filtered_count() as u16).min(max_list_height).max(1);
    let popup_height = list_height + 3; // +1 header, +2 border

    // Position: centered horizontally, upper third vertically
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + area.height / 4;
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

    // Build list items
    let items: Vec<ListItem> = state
        .filtered_indices
        .iter()
        .enumerate()
        .take(list_area.height as usize) // Only render what fits
        .map(|(i, &opt_idx)| {
            let opt = &state.all_options[opt_idx];
            let style = if i == state.selected_index {
                Style::default()
                    .fg(theme.local_branch)
                    .add_modifier(Modifier::BOLD)
                    .bg(theme.selection_bg)
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(opt.clone(), style))
        })
        .collect();

    // Show "No matches" if list is empty
    if items.is_empty() {
        let no_matches = Paragraph::new(Span::styled(
            "No matches",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(no_matches, list_area);
    } else {
        let list = List::new(items);
        frame.render_widget(list, list_area);
    }
}
