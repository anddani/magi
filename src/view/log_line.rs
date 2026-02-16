use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::{config::Theme, model::LogEntry};

/// Get the display lines for a log entry
pub fn get_lines(entry: &LogEntry, theme: &Theme) -> Vec<Line<'static>> {
    let mut spans: Vec<Span> = Vec::new();

    // Graph portion - colored
    if !entry.graph.is_empty() {
        spans.push(Span::styled(
            entry.graph.clone(),
            Style::default().fg(theme.diff_context),
        ));
    }

    // Hash
    if let Some(ref hash) = entry.hash {
        spans.push(Span::styled(
            hash.clone(),
            Style::default()
                .fg(theme.commit_hash)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" "));
    }

    // Refs (branches, tags)
    if let Some(ref refs) = entry.refs {
        let ref_spans = parse_refs(refs, theme);
        spans.extend(ref_spans);
        spans.push(Span::raw(" "));
    }

    // Message
    if let Some(ref message) = entry.message {
        spans.push(Span::styled(message.clone(), Style::default()));
    }

    // Author and time (dimmed)
    if let Some(ref author) = entry.author
        && let Some(ref time) = entry.time
    {
        spans.push(Span::styled(
            format!(" - {} ({})", author, time),
            Style::default().fg(Color::DarkGray),
        ));
    }

    vec![Line::from(spans)]
}

/// Parse refs string and return colored spans
fn parse_refs(refs: &str, theme: &Theme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    // refs can contain multiple references separated by ", "
    // e.g., "HEAD -> main, origin/main, tag: v1.0"
    let parts: Vec<&str> = refs.split(", ").collect();

    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(", "));
        }

        let part = part.trim();

        if part.starts_with("HEAD -> ") {
            // HEAD pointing to a branch
            spans.push(Span::styled(
                "HEAD -> ".to_string(),
                Style::default().fg(theme.commit_hash),
            ));
            let branch_name = part.strip_prefix("HEAD -> ").unwrap_or(part);
            spans.push(Span::styled(
                branch_name.to_string(),
                Style::default()
                    .fg(theme.local_branch)
                    .add_modifier(Modifier::BOLD),
            ));
        } else if part == "HEAD" {
            spans.push(Span::styled(
                "HEAD".to_string(),
                Style::default().fg(theme.commit_hash),
            ));
        } else if part.starts_with("tag: ") {
            // Tag
            spans.push(Span::styled(
                part.to_string(),
                Style::default().fg(theme.tag_label),
            ));
        } else if part.contains('/') {
            // Remote branch
            spans.push(Span::styled(
                part.to_string(),
                Style::default().fg(theme.remote_branch),
            ));
        } else {
            // Local branch
            spans.push(Span::styled(
                part.to_string(),
                Style::default()
                    .fg(theme.local_branch)
                    .add_modifier(Modifier::BOLD),
            ));
        }
    }

    if !spans.is_empty() {
        // Wrap refs in parentheses
        spans.insert(0, Span::raw("("));
        spans.push(Span::raw(")"));
    }

    spans
}
