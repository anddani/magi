use ratatui::text::Line as TextLine;

use super::util::format_ref_with_colors;
use crate::config::Theme;
use crate::git::GitRef;

/// Generate the view lines for a head reference
pub fn get_lines(head_ref: &GitRef, collapsed: bool, theme: &Theme) -> Vec<TextLine<'static>> {
    let indicator = if collapsed { ">" } else { "∨" };
    let prefix = format!("{}Head:     ", indicator);
    let ref_with_color = format_ref_with_colors(head_ref, &prefix, theme);

    vec![TextLine::from(ref_with_color)]
}
