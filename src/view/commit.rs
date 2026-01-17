use ratatui::{
    style::Style,
    text::{Line as TextLine, Span},
};

use crate::config::Theme;
use crate::git::CommitInfo;

/// Generate the view lines for a commit
pub fn get_lines(commit: &CommitInfo, theme: &Theme) -> Vec<TextLine<'static>> {
    let mut spans = Vec::new();

    // Indentation
    spans.push(Span::raw(" "));

    // Commit hash (7 chars) - use commit_hash color
    spans.push(Span::styled(
        commit.hash.clone(),
        Style::default().fg(theme.commit_hash),
    ));

    // Branch name (if present)
    if let Some(ref branch) = commit.branch {
        spans.push(Span::raw(" "));
        if branch == "@" {
            // Detached head - use detached_head color
            spans.push(Span::styled(
                branch.clone(),
                Style::default().fg(theme.detached_head),
            ));
        } else {
            // Local branch - use local_branch color
            spans.push(Span::styled(
                branch.clone(),
                Style::default().fg(theme.local_branch),
            ));
        }
    }

    // Upstream branch (if present)
    if let Some(ref upstream) = commit.upstream {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            upstream.clone(),
            Style::default().fg(theme.remote_branch),
        ));
    }

    // Tag (if present)
    if let Some(ref tag) = commit.tag {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            tag.clone(),
            Style::default().fg(theme.tag_label),
        ));
    }

    // Commit message - use text color
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        commit.message.clone(),
        Style::default().fg(theme.text),
    ));

    vec![TextLine::from(spans)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_theme() -> Theme {
        Theme::default()
    }

    #[test]
    fn test_basic_commit_rendering() {
        let commit = CommitInfo {
            hash: "abc1234".to_string(),
            branch: None,
            upstream: None,
            tag: None,
            message: "Initial commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme);

        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_commit_with_branch() {
        let commit = CommitInfo {
            hash: "abc1234".to_string(),
            branch: Some("main".to_string()),
            upstream: None,
            tag: None,
            message: "Initial commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme);

        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_commit_with_all_info() {
        let commit = CommitInfo {
            hash: "abc1234".to_string(),
            branch: Some("main".to_string()),
            upstream: Some("origin/main".to_string()),
            tag: Some("v1.0.0".to_string()),
            message: "Release commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme);

        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_detached_head_commit() {
        let commit = CommitInfo {
            hash: "abc1234".to_string(),
            branch: Some("@".to_string()),
            upstream: None,
            tag: None,
            message: "Detached commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme);

        assert_eq!(lines.len(), 1);
    }
}
