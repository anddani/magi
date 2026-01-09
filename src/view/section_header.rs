use ratatui::{
    style::Style,
    text::{Line as TextLine, Span},
};

use crate::config::Theme;

/// Generate the view lines for a section header
pub fn get_lines(
    title: &str,
    count: Option<usize>,
    collapsed: bool,
    theme: &Theme,
) -> Vec<TextLine<'static>> {
    // Create the section header line with expand/collapse indicator
    // Use '>' when collapsed, '∨' when expanded
    let indicator = if collapsed { ">" } else { "∨" };
    let header_text = if let Some(count) = count {
        format!("{}{} ({})", indicator, title, count)
    } else {
        format!("{}{}", indicator, title)
    };

    let header_line = TextLine::from(vec![Span::styled(
        header_text,
        Style::default().fg(theme.section_header),
    )]);

    vec![header_line]
}
