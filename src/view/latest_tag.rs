use ratatui::{
    style::Style,
    text::{Line as TextLine, Span},
};

use crate::config::Theme;
use crate::git::TagInfo;

/// Generate the view lines for latest tag
pub fn get_lines(tag_info: &TagInfo, theme: &Theme) -> Vec<TextLine<'static>> {
    vec![TextLine::from(vec![
        Span::styled(" Tag:      ", Style::default().fg(theme.tag_label)),
        Span::styled(tag_info.name.clone(), Style::default().fg(theme.tag_label)),
        Span::styled(" (", Style::default().fg(theme.text)),
        Span::styled(
            tag_info.commits_ahead.to_string(),
            Style::default().fg(theme.tag_label),
        ),
        Span::styled(")", Style::default().fg(theme.text)),
    ])]
}
