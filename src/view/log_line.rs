use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
};

use crate::{config::Theme, git::CommitRefType, model::LogEntry};

/// Get the display lines for a log entry
/// If current_branch is provided, that branch will be highlighted with inverted colors
pub fn get_lines(
    entry: &LogEntry,
    theme: &Theme,
    is_detached_head: bool,
    current_branch: Option<&str>,
) -> Vec<Line<'static>> {
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
        for commit_ref in entry.refs.iter() {
            if commit_ref.ref_type == CommitRefType::Head && !is_detached_head {
                continue;
            }
            let color = match commit_ref.ref_type {
                CommitRefType::Head => theme.detached_head,
                CommitRefType::LocalBranch => theme.local_branch,
                CommitRefType::RemoteBranch => theme.remote_branch,
                CommitRefType::Tag => theme.tag_label,
            };
            // Invert colors for checked out branch (color as background, dark text)
            let style = if current_branch == Some(commit_ref.name.as_str()) {
                Style::default().fg(color).underlined().bold()
            } else {
                Style::default().fg(color)
            };
            spans.push(Span::styled(commit_ref.name.clone(), style));
            spans.push(Span::raw(" "));
        }
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
            format!(" - {} {}", author, time),
            Style::default().fg(Color::DarkGray),
        ));
    }

    vec![Line::from(spans)]
}
