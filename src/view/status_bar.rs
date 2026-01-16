use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line as TextLine, Span},
    widgets::Paragraph,
    Frame,
};

use crate::{config::Theme, model::InputMode};

/// Render the status bar at the bottom of the screen
///
/// Layout:
/// - Left: Mode indicator (NORMAL, VISUAL, SEARCH)
/// - Center: Search text (only in Search mode)
/// - Right: Directory path (hidden in Search mode)
pub fn render_status_bar(
    frame: &mut Frame,
    area: Rect,
    mode: InputMode,
    search_query: &str,
    directory: &str,
    theme: &Theme,
) {
    let width = area.width as usize;

    // Get mode-specific colors
    let (mode_bg, mode_fg) = match mode {
        InputMode::Normal => (theme.status_mode_normal_bg, theme.status_mode_normal_fg),
        InputMode::Visual => (theme.status_mode_visual_bg, theme.status_mode_visual_fg),
        InputMode::Search => (theme.status_mode_search_bg, theme.status_mode_search_fg),
    };

    let mode_text = format!(" {} ", mode.display_name());
    let mode_span = Span::styled(mode_text.clone(), Style::default().fg(mode_fg).bg(mode_bg));

    let mut spans = vec![mode_span];

    if mode == InputMode::Search {
        // In search mode: show search query, hide directory
        let search_text = format!(" /{}", search_query);
        spans.push(Span::styled(
            search_text,
            Style::default()
                .fg(theme.status_bar_fg)
                .bg(theme.status_bar_bg),
        ));

        // Fill remaining space with background
        let used_width = mode_text.len() + 2 + search_query.len();
        let remaining = width.saturating_sub(used_width);
        if remaining > 0 {
            spans.push(Span::styled(
                " ".repeat(remaining),
                Style::default().bg(theme.status_bar_bg),
            ));
        }
    } else {
        // Normal/Visual mode: show directory on the right
        let dir_display = format!(" {} ", directory);
        let mode_width = mode_text.len();
        let dir_width = dir_display.len();

        // Calculate padding to push directory to the right
        let padding_width = width.saturating_sub(mode_width + dir_width);

        spans.push(Span::styled(
            " ".repeat(padding_width),
            Style::default().bg(theme.status_bar_bg),
        ));

        spans.push(Span::styled(
            dir_display,
            Style::default()
                .fg(theme.status_bar_fg)
                .bg(theme.status_bar_bg),
        ));
    }

    let line = TextLine::from(spans);
    let paragraph = Paragraph::new(line);

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_mode_display_name() {
        assert_eq!(InputMode::Normal.display_name(), "NORMAL");
        assert_eq!(InputMode::Visual.display_name(), "VISUAL");
        assert_eq!(InputMode::Search.display_name(), "SEARCH");
    }
}
