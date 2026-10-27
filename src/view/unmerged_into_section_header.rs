use ratatui::{
    style::Style,
    text::{Line as TextLine, Span},
};

use crate::{config::Theme, i18n};

/// Generate the view lines for the "Unmerged into [branch]" section header
/// The branch name is colored with the remote_branch color, the rest with section_header color
pub fn get_lines(
    branch_name: &str,
    count: usize,
    collapsed: bool,
    theme: &Theme,
) -> Vec<TextLine<'static>> {
    let indicator = if collapsed { ">" } else { "∨" };

    let header_line = TextLine::from(vec![
        Span::styled(
            format!("{}{}", indicator, i18n::t().section_unmerged_into_prefix),
            Style::default().fg(theme.section_header),
        ),
        Span::styled(
            branch_name.to_string(),
            Style::default().fg(theme.remote_branch),
        ),
        Span::styled(format!(" ({})", count), Style::default().fg(theme.text)),
    ]);

    vec![header_line]
}
