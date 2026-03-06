use ratatui::{
    style::Style,
    text::{Line as TextLine, Span},
};

use crate::config::Theme;
use crate::model::{DiffLine, DiffLineType};
use crate::view::util::expand_tabs;

/// Generate the view lines for a diff line
pub fn get_lines(diff_line: &DiffLine, theme: &Theme) -> Vec<TextLine<'static>> {
    let (prefix, color) = match diff_line.line_type {
        DiffLineType::Addition => ("+", theme.diff_addition),
        DiffLineType::Deletion => ("-", theme.diff_deletion),
        DiffLineType::Context => (" ", theme.diff_context),
    };

    // The prefix spans are 2 columns wide (" " + prefix char), so content
    // tab stops should be calculated from column 2.
    let content = expand_tabs(&diff_line.content, 2);

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
