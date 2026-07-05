use ratatui::{
    style::Style,
    text::{Line as TextLine, Span},
};

use crate::{
    config::Theme,
    git::rebase::{RebaseAction, RebaseTodoEntry},
};

/// Render a single line in the interactive rebase todo editor.
/// Format: `action  hash message`, with the action word colour-coded.
pub fn get_lines(entry: &RebaseTodoEntry, theme: &Theme) -> Vec<TextLine<'static>> {
    let action_style = match entry.action {
        RebaseAction::Pick => Style::default().fg(theme.text),
        RebaseAction::Reword => Style::default().fg(theme.local_branch),
        RebaseAction::Edit => Style::default().fg(theme.section_header),
        RebaseAction::Squash | RebaseAction::Fixup => Style::default().fg(theme.remote_branch),
        RebaseAction::Drop => Style::default().fg(theme.diff_deletion),
    };

    let short_hash: String = entry.hash.chars().take(7).collect();

    let mut spans = vec![
        Span::raw(" "),
        // Pad to the longest action word ("reword"/"squash") plus one space
        Span::styled(format!("{:<7}", entry.action.as_str()), action_style),
        Span::styled(short_hash, Style::default().fg(theme.commit_hash)),
    ];

    if !entry.message.is_empty() {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            entry.message.clone(),
            Style::default().fg(theme.text),
        ));
    }

    vec![TextLine::from(spans)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(action: RebaseAction) -> RebaseTodoEntry {
        RebaseTodoEntry {
            action,
            hash: "abc1234def5678".to_string(),
            message: "Fix bug".to_string(),
        }
    }

    fn get_span_texts(line: &TextLine) -> Vec<String> {
        line.spans.iter().map(|s| s.content.to_string()).collect()
    }

    #[test]
    fn test_shows_action_word_and_short_hash() {
        let theme = Theme::default();
        let lines = get_lines(&entry(RebaseAction::Squash), &theme);
        assert_eq!(lines.len(), 1);
        let texts = get_span_texts(&lines[0]);
        assert!(texts.iter().any(|s| s.starts_with("squash")));
        assert!(texts.iter().any(|s| s == "abc1234"));
        assert!(texts.iter().any(|s| s == "Fix bug"));
    }

    #[test]
    fn test_drop_uses_deletion_color() {
        let theme = Theme::default();
        let lines = get_lines(&entry(RebaseAction::Drop), &theme);
        let action_span = lines[0]
            .spans
            .iter()
            .find(|s| s.content.starts_with("drop"))
            .unwrap();
        assert_eq!(action_span.style.fg, Some(theme.diff_deletion));
    }

    #[test]
    fn test_action_words_are_equal_width() {
        let theme = Theme::default();
        for action in [
            RebaseAction::Pick,
            RebaseAction::Reword,
            RebaseAction::Edit,
            RebaseAction::Squash,
            RebaseAction::Fixup,
            RebaseAction::Drop,
        ] {
            let lines = get_lines(&entry(action), &theme);
            // Span 1 is the padded action word
            assert_eq!(lines[0].spans[1].content.len(), 7);
        }
    }
}
