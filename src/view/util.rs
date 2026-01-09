use ratatui::{
    style::{Color, Style},
    text::{Line as TextLine, Span},
};

use crate::config::Theme;
use crate::git::{GitRef, ReferenceType};
use crate::model::{FileChange, FileStatus};

/// Format a GitRef with appropriate colors for different parts using Ratatui's styling system
pub fn format_ref_with_colors<'a>(ref_info: &GitRef, label: &str, theme: &Theme) -> Vec<Span<'a>> {
    let mut spans = vec![Span::styled(
        label.to_string(),
        Style::default().fg(theme.ref_label),
    )];

    // Branch name with appropriate color
    let branch_style = match ref_info.reference_type {
        ReferenceType::RemoteBranch => Style::default().fg(theme.remote_branch),
        ReferenceType::DetachedHead => Style::default().fg(theme.detached_head),
        ReferenceType::LocalBranch => Style::default().fg(theme.local_branch),
    };

    spans.push(Span::styled(ref_info.name.clone(), branch_style));
    spans.push(Span::styled(" ", Style::default()));

    // Git hash
    spans.push(Span::styled(
        ref_info.commit_hash.clone(),
        Style::default().fg(theme.commit_hash),
    ));

    spans.push(Span::styled(" ", Style::default()));

    // Commit message
    spans.push(Span::styled(
        ref_info.commit_message.clone(),
        Style::default().fg(theme.text),
    ));

    spans
}

/// Generate the view line for a file change (staged or unstaged)
pub fn format_file_change(
    file_change: &FileChange,
    collapsed: bool,
    status_color: Color,
    theme: &Theme,
) -> TextLine<'static> {
    let status_str = match file_change.status {
        FileStatus::Modified => "modified",
        FileStatus::Deleted => "deleted",
        FileStatus::New => "new file",
        FileStatus::Renamed => "renamed",
        FileStatus::Copied => "copied",
        FileStatus::TypeChange => "typechange",
    };

    // Use '>' when collapsed, '∨' when expanded
    let indicator = if collapsed { ">" } else { "∨" };

    TextLine::from(vec![
        Span::raw(indicator),
        Span::styled(
            format!("{} ", status_str),
            Style::default().fg(status_color),
        ),
        Span::styled(
            file_change.path.clone(),
            Style::default().fg(theme.file_path),
        ),
    ])
}
