use ratatui::{
    style::Style,
    text::{Line as TextLine, Span},
};

use crate::config::Theme;
use crate::git::StashEntry;

/// Generate the view line for a stash entry.
/// Displays the stash index (stash@{N}) followed by the stash message.
pub fn get_lines(stash: &StashEntry, theme: &Theme) -> Vec<TextLine<'static>> {
    let mut spans = Vec::new();

    spans.push(Span::raw(" "));

    // Stash ID in commit_hash color, matching how magit highlights the stash designator
    spans.push(Span::styled(
        format!("stash@{{{}}}", stash.index),
        Style::default().fg(theme.commit_hash),
    ));

    spans.push(Span::raw(" "));

    spans.push(Span::styled(
        stash.message.clone(),
        Style::default().fg(theme.text),
    ));

    vec![TextLine::from(spans)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::StashEntry;

    fn test_theme() -> Theme {
        Theme::default()
    }

    #[test]
    fn test_stash_line_contains_index_and_message() {
        let stash = StashEntry {
            index: 0,
            message: "WIP on main: abc1234 Initial commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&stash, &theme);

        assert_eq!(lines.len(), 1);
        let content: Vec<String> = lines[0]
            .spans
            .iter()
            .map(|s| s.content.to_string())
            .collect();
        assert!(content.iter().any(|s| s == "stash@{0}"));
        assert!(
            content
                .iter()
                .any(|s| s == "WIP on main: abc1234 Initial commit")
        );
    }

    #[test]
    fn test_stash_line_index_uses_commit_hash_color() {
        let stash = StashEntry {
            index: 2,
            message: "On main: my stash".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&stash, &theme);

        let id_span = lines[0]
            .spans
            .iter()
            .find(|s| s.content == "stash@{2}")
            .unwrap();
        assert_eq!(id_span.style.fg, Some(theme.commit_hash));
    }
}
