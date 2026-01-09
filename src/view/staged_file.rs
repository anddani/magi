use ratatui::text::Line as TextLine;

use crate::config::Theme;
use crate::model::FileChange;

use super::util::format_file_change;

/// Generate the view lines for a staged file change
pub fn get_lines(
    file_change: &FileChange,
    collapsed: bool,
    theme: &Theme,
) -> Vec<TextLine<'static>> {
    vec![format_file_change(
        file_change,
        collapsed,
        theme.staged_status,
        theme,
    )]
}
