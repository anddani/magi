use ratatui::text::Line as TextLine;

use super::util::format_ref_with_colors;
use crate::config::Theme;
use crate::git::GitRef;
use crate::i18n;

/// Generate the view lines for a push remote reference
pub fn get_lines(push_ref: &GitRef, theme: &Theme) -> Vec<TextLine<'static>> {
    let prefix = format!(" {}", i18n::t().ref_push);
    vec![TextLine::from(format_ref_with_colors(
        push_ref, &prefix, theme,
    ))]
}
