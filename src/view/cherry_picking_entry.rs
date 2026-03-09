use ratatui::{
    style::Style,
    text::{Line as TextLine, Span},
};

use crate::config::Theme;

/// Render a single line in the "Cherry Picking" sequencer section.
///
/// - `is_current = true`  → the commit currently stopped on (shows "stop" in deletion color)
/// - `is_current = false` → a pending commit in the sequencer (shows "pick" in normal text color)
pub fn get_lines(
    hash: &str,
    message: &str,
    is_current: bool,
    theme: &Theme,
) -> Vec<TextLine<'static>> {
    let mut spans = Vec::new();

    // Indentation
    spans.push(Span::raw(" "));

    // Label: "stop" for the current stopped commit, "pick" for pending
    let (label, label_style) = if is_current {
        ("stop ", Style::default().fg(theme.diff_deletion))
    } else {
        ("pick ", Style::default().fg(theme.text))
    };
    spans.push(Span::styled(label.to_string(), label_style));

    // Short hash
    spans.push(Span::styled(
        hash.to_string(),
        Style::default().fg(theme.commit_hash),
    ));

    // Commit message
    if !message.is_empty() {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            message.to_string(),
            Style::default().fg(theme.text),
        ));
    }

    vec![TextLine::from(spans)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_theme() -> Theme {
        Theme::default()
    }

    fn get_span_texts(line: &TextLine) -> Vec<String> {
        line.spans.iter().map(|s| s.content.to_string()).collect()
    }

    #[test]
    fn test_current_entry_shows_stop_label() {
        let theme = test_theme();
        let lines = get_lines("abc1234", "Fix bug", true, &theme);
        assert_eq!(lines.len(), 1);
        let texts = get_span_texts(&lines[0]);
        assert!(texts.iter().any(|s| s.contains("stop")));
        assert!(!texts.iter().any(|s| s.contains("pick")));
    }

    #[test]
    fn test_pending_entry_shows_pick_label() {
        let theme = test_theme();
        let lines = get_lines("abc1234", "Fix bug", false, &theme);
        assert_eq!(lines.len(), 1);
        let texts = get_span_texts(&lines[0]);
        assert!(texts.iter().any(|s| s.contains("pick")));
        assert!(!texts.iter().any(|s| s.contains("stop")));
    }

    #[test]
    fn test_includes_hash() {
        let theme = test_theme();
        let lines = get_lines("abc1234", "Fix bug", true, &theme);
        let texts = get_span_texts(&lines[0]);
        assert!(texts.iter().any(|s| s == "abc1234"));
    }

    #[test]
    fn test_includes_message() {
        let theme = test_theme();
        let lines = get_lines("abc1234", "Fix bug", false, &theme);
        let texts = get_span_texts(&lines[0]);
        assert!(texts.iter().any(|s| s == "Fix bug"));
    }

    #[test]
    fn test_current_entry_uses_deletion_color() {
        let theme = test_theme();
        let lines = get_lines("abc1234", "Fix bug", true, &theme);
        let stop_span = lines[0]
            .spans
            .iter()
            .find(|s| s.content.contains("stop"))
            .unwrap();
        assert_eq!(stop_span.style.fg, Some(theme.diff_deletion));
    }

    #[test]
    fn test_hash_uses_commit_hash_color() {
        let theme = test_theme();
        let lines = get_lines("abc1234", "Fix bug", false, &theme);
        let hash_span = lines[0]
            .spans
            .iter()
            .find(|s| s.content == "abc1234")
            .unwrap();
        assert_eq!(hash_span.style.fg, Some(theme.commit_hash));
    }
}
