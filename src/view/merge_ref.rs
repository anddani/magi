use ratatui::text::Line as TextLine;

use super::util::format_ref_with_colors;
use crate::config::Theme;
use crate::git::GitRef;

/// Generate the view lines for a merge (upstream) reference
pub fn get_lines(merge_ref: &GitRef, theme: &Theme) -> Vec<TextLine<'static>> {
    vec![TextLine::from(format_ref_with_colors(
        merge_ref,
        " Merge:   ",
        theme,
    ))]
}
