use ratatui::{
    style::Style,
    text::{Line as TextLine, Span},
};

use crate::config::Theme;
use crate::model::{DiffLine, DiffLineType};

/// Generate the view lines for a diff line
pub fn get_lines(diff_line: &DiffLine, theme: &Theme) -> Vec<TextLine<'static>> {
    let (prefix, color) = match diff_line.line_type {
        DiffLineType::Addition => ("+", theme.diff_addition),
        DiffLineType::Deletion => ("-", theme.diff_deletion),
        DiffLineType::Context => (" ", theme.diff_context),
    };

    let line = TextLine::from(vec![
        Span::raw(" "),
        Span::styled(prefix.to_string(), Style::default().fg(color)),
        Span::styled(diff_line.content.clone(), Style::default().fg(color)),
    ]);

    vec![line]
}
