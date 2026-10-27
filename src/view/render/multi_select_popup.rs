use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::config::Theme;
use crate::model::popup::MultiSelectPopupState;
use crate::view::render::util::input_spans;

/// Render the multi-select popup as a bottom sheet (like the select popup),
/// prefixing each row with a `[x]`/`[ ]` checkbox.
pub fn render(state: &MultiSelectPopupState, frame: &mut Frame, area: Rect, theme: &Theme) {
    let popup_width = area.width;
    let popup_height = (area.height / 4).max(5);
    let x = area.x;
    let y = area.y + area.height.saturating_sub(popup_height);
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(state.title.as_str())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.local_branch));
    frame.render_widget(block, popup_area);

    let inner = Rect::new(
        popup_area.x + 1,
        popup_area.y + 1,
        popup_area.width.saturating_sub(2),
        popup_area.height.saturating_sub(2),
    );

    // Header line: "[x]/[y] (n) Title: input_"
    let display_index = if state.filtered_count() > 0 {
        state.selected_index + 1
    } else {
        0
    };
    let header_prefix = format!(
        "[{}]/[{}] ({}) {}: ",
        display_index,
        state.filtered_count(),
        state.checked.len(),
        state.title,
    );
    let mut header_spans = vec![Span::styled(header_prefix, Style::default())];
    header_spans.extend(input_spans(state.input.as_str(), state.input.cursor()));
    let header_line = Line::from(header_spans);
    let header_area = Rect::new(inner.x, inner.y, inner.width, 1);
    frame.render_widget(Paragraph::new(header_line), header_area);

    let list_area = Rect::new(
        inner.x,
        inner.y + 1,
        inner.width,
        inner.height.saturating_sub(1),
    );

    let visible_count = list_area.height as usize;
    let scroll_offset = if state.selected_index >= visible_count {
        state.selected_index - visible_count + 1
    } else {
        0
    };

    let items: Vec<ListItem> = state
        .filtered_indices
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_count)
        .map(|(i, &opt_idx)| {
            let opt = &state.all_options[opt_idx];
            let checkbox = if state.is_checked(opt_idx) {
                "[x] "
            } else {
                "[ ] "
            };
            let style = if i == state.selected_index {
                Style::default()
                    .fg(theme.local_branch)
                    .add_modifier(Modifier::BOLD)
                    .bg(theme.selection_bg)
            } else if state.is_checked(opt_idx) {
                Style::default().fg(theme.diff_addition)
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(format!("{checkbox}{opt}"), style))
        })
        .collect();

    if items.is_empty() {
        let no_matches = Paragraph::new(Span::styled(
            "No matches",
            Style::default().fg(theme.dim_text),
        ));
        frame.render_widget(no_matches, list_area);
    } else {
        let list = List::new(items);
        frame.render_widget(list, list_area);
    }
}
