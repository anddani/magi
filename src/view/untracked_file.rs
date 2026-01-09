use ratatui::{
    style::Style,
    text::{Line as TextLine, Span},
};

use crate::config::Theme;

/// Generate the view lines for an untracked file
pub fn get_lines(file_path: &str, theme: &Theme) -> Vec<TextLine<'static>> {
    // Create the untracked file line with proper indentation
    let file_line = TextLine::from(vec![
        Span::raw(" "), // Indentation space
        Span::styled(
            file_path.to_string(),
            Style::default().fg(theme.untracked_file),
        ),
    ]);

    vec![file_line]
}
