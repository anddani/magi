use ratatui::{
    style::Style,
    text::{Line as TextLine, Span},
};

use crate::config::Theme;
use crate::model::{DiffLine, DiffLineType};
use crate::view::util::expand_tabs;

/// Generate the view lines for a diff line
pub fn get_lines(diff_line: &DiffLine, theme: &Theme) -> Vec<TextLine<'static>> {
    // Combined-diff lines already carry their origin prefix in `content`
    let (prefix, color) = match diff_line.line_type {
        DiffLineType::Addition => ("+", theme.diff_addition),
        DiffLineType::Deletion => ("-", theme.diff_deletion),
        DiffLineType::Context => (" ", theme.diff_context),
        DiffLineType::CombinedAddition => ("", theme.diff_addition),
        DiffLineType::CombinedDeletion => ("", theme.diff_deletion),
        DiffLineType::CombinedContext => ("", theme.diff_context),
        DiffLineType::ConflictMarker => ("", theme.diff_hunk),
    };

    // Content tab stops are calculated from the column where the content
    // starts: the leading space plus the prefix span.
    let content = expand_tabs(&diff_line.content, 1 + prefix.len());

    let line = TextLine::from(vec![
        Span::raw(" "),
        Span::styled(prefix.to_string(), Style::default().fg(color)),
        Span::styled(content, Style::default().fg(color)),
    ]);

    vec![line]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_tabs_no_tabs() {
        assert_eq!(expand_tabs("hello world", 0), "hello world");
    }

    #[test]
    fn expand_tabs_single_tab_at_start() {
        // Tab at col 0 → 4 spaces
        assert_eq!(expand_tabs("\thello", 0), "    hello");
    }

    #[test]
    fn expand_tabs_tab_mid_string() {
        // "ab\t" at col 0: "ab" takes 2 cols, tab aligns to next 4-stop → 2 spaces
        assert_eq!(expand_tabs("ab\tc", 0), "ab  c");
    }

    #[test]
    fn expand_tabs_with_initial_col_offset() {
        // With initial_col=2 (like real diff rendering), tab at start aligns to col 4 → 2 spaces
        assert_eq!(expand_tabs("\thello", 2), "  hello");
    }

    #[test]
    fn expand_tabs_multiple_tabs() {
        assert_eq!(expand_tabs("\t\t", 0), "        ");
    }
}
