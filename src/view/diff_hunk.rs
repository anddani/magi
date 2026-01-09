use ratatui::{
    style::Style,
    text::{Line as TextLine, Span},
};

use crate::config::Theme;
use crate::model::DiffHunk;

/// Generate the view lines for a diff hunk header
pub fn get_lines(hunk: &DiffHunk, theme: &Theme) -> Vec<TextLine<'static>> {
    let hunk_line = TextLine::from(vec![
        Span::raw(" "),
        Span::styled(hunk.header.clone(), Style::default().fg(theme.diff_hunk)),
    ]);

    vec![hunk_line]
}
