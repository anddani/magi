use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::{config::Theme, git::CommitRefType, model::LogEntry};

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
    if !entry.refs.is_empty() {
        spans.push(Span::raw("("));
        for (i, commit_ref) in entry.refs.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            let color = match commit_ref.ref_type {
                CommitRefType::Head => theme.detached_head,
                CommitRefType::LocalBranch => theme.local_branch,
                CommitRefType::RemoteBranch => theme.remote_branch,
                CommitRefType::Tag => theme.tag_label,
            };
            spans.push(Span::styled(
                commit_ref.name.clone(),
                Style::default().fg(color),
            ));
        }
        spans.push(Span::raw(") "));
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
