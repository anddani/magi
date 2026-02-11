use ratatui::{
    style::Style,
    text::{Line as TextLine, Span},
};

use crate::config::Theme;

/// Generate the view lines for the "Unpulled from [remote]" section header
/// The remote name is colored with the remote_branch color, the rest with section_header color
pub fn get_lines(
    remote_name: &str,
    count: usize,
    collapsed: bool,
    theme: &Theme,
) -> Vec<TextLine<'static>> {
    let indicator = if collapsed { ">" } else { "∨" };

    let header_line = TextLine::from(vec![
        Span::styled(
            format!("{}Unpulled from ", indicator),
            Style::default().fg(theme.section_header),
        ),
        Span::styled(
            remote_name.to_string(),
            Style::default().fg(theme.remote_branch),
        ),
        Span::styled(format!(" ({})", count), Style::default().fg(theme.text)),
    ]);

    vec![header_line]
}
