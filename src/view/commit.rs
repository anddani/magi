use ratatui::{
    style::Style,
    text::{Line as TextLine, Span},
};

use crate::config::Theme;
use crate::git::{CommitInfo, CommitRefType};

/// Generate the view lines for a commit
/// If current_branch is provided, that branch will be highlighted with a background color
pub fn get_lines(
    commit: &CommitInfo,
    theme: &Theme,
    current_branch: Option<&str>,
) -> Vec<TextLine<'static>> {
    let mut spans = Vec::new();

    // Indentation
    spans.push(Span::raw(" "));

    // Commit hash (7 chars) - use commit_hash color
    spans.push(Span::styled(
        commit.hash.clone(),
        Style::default().fg(theme.commit_hash),
    ));

    // All refs (branches, tags) in order: current branch, HEAD/@, other local, remote, tags
    for commit_ref in &commit.refs {
        spans.push(Span::raw(" "));
        let color = match commit_ref.ref_type {
            CommitRefType::Head => theme.detached_head,
            CommitRefType::LocalBranch => theme.local_branch,
            CommitRefType::RemoteBranch => theme.remote_branch,
            CommitRefType::Tag => theme.tag_label,
        };
        // Invert colors for checked out branch (color as background, dark text)
        let style = if current_branch == Some(commit_ref.name.as_str()) {
            Style::default().bg(color).fg(theme.checked_out_branch_bg)
        } else {
            Style::default().fg(color)
        };
        spans.push(Span::styled(commit_ref.name.clone(), style));
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
    use crate::git::CommitRef;

    fn test_theme() -> Theme {
        Theme::default()
    }

    fn get_span_content(line: &TextLine) -> Vec<String> {
        line.spans.iter().map(|s| s.content.to_string()).collect()
    }

    #[test]
    fn test_basic_commit_has_hash_and_message() {
        let commit = CommitInfo {
            hash: "abc1234".to_string(),
            refs: vec![],
            message: "Initial commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme, None);

        assert_eq!(lines.len(), 1);
        let content = get_span_content(&lines[0]);
        assert!(content.iter().any(|s| s == "abc1234"));
        assert!(content.iter().any(|s| s == "Initial commit"));
    }

    #[test]
    fn test_commit_with_branch_includes_branch_name() {
        let commit = CommitInfo {
            hash: "abc1234".to_string(),
            refs: vec![CommitRef {
                name: "main".to_string(),
                ref_type: CommitRefType::LocalBranch,
            }],
            message: "Initial commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme, None);

        let content = get_span_content(&lines[0]);
        assert!(content.iter().any(|s| s == "main"));
    }

    #[test]
    fn test_commit_with_all_info_includes_all_elements() {
        let commit = CommitInfo {
            hash: "abc1234".to_string(),
            refs: vec![
                CommitRef {
                    name: "main".to_string(),
                    ref_type: CommitRefType::LocalBranch,
                },
                CommitRef {
                    name: "origin/main".to_string(),
                    ref_type: CommitRefType::RemoteBranch,
                },
                CommitRef {
                    name: "v1.0.0".to_string(),
                    ref_type: CommitRefType::Tag,
                },
            ],
            message: "Release commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme, None);

        let content = get_span_content(&lines[0]);
        assert!(content.iter().any(|s| s == "abc1234"));
        assert!(content.iter().any(|s| s == "main"));
        assert!(content.iter().any(|s| s == "origin/main"));
        assert!(content.iter().any(|s| s == "v1.0.0"));
        assert!(content.iter().any(|s| s == "Release commit"));
    }

    #[test]
    fn test_detached_head_shows_at_symbol() {
        let commit = CommitInfo {
            hash: "abc1234".to_string(),
            refs: vec![CommitRef {
                name: "@".to_string(),
                ref_type: CommitRefType::Head,
            }],
            message: "Detached commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme, None);

        let content = get_span_content(&lines[0]);
        assert!(content.iter().any(|s| s == "@"));
    }

    #[test]
    fn test_detached_head_uses_detached_head_color() {
        let commit = CommitInfo {
            hash: "abc1234".to_string(),
            refs: vec![CommitRef {
                name: "@".to_string(),
                ref_type: CommitRefType::Head,
            }],
            message: "Detached commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme, None);

        let at_span = lines[0].spans.iter().find(|s| s.content == "@").unwrap();
        assert_eq!(at_span.style.fg, Some(theme.detached_head));
    }

    #[test]
    fn test_local_branch_uses_local_branch_color() {
        let commit = CommitInfo {
            hash: "abc1234".to_string(),
            refs: vec![CommitRef {
                name: "main".to_string(),
                ref_type: CommitRefType::LocalBranch,
            }],
            message: "Commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme, None);

        let branch_span = lines[0].spans.iter().find(|s| s.content == "main").unwrap();
        assert_eq!(branch_span.style.fg, Some(theme.local_branch));
    }

    #[test]
    fn test_remote_branch_uses_remote_branch_color() {
        let commit = CommitInfo {
            hash: "abc1234".to_string(),
            refs: vec![CommitRef {
                name: "origin/main".to_string(),
                ref_type: CommitRefType::RemoteBranch,
            }],
            message: "Commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme, None);

        let branch_span = lines[0]
            .spans
            .iter()
            .find(|s| s.content == "origin/main")
            .unwrap();
        assert_eq!(branch_span.style.fg, Some(theme.remote_branch));
    }

    #[test]
    fn test_hash_uses_commit_hash_color() {
        let commit = CommitInfo {
            hash: "abc1234".to_string(),
            refs: vec![],
            message: "Commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme, None);

        let hash_span = lines[0]
            .spans
            .iter()
            .find(|s| s.content == "abc1234")
            .unwrap();
        assert_eq!(hash_span.style.fg, Some(theme.commit_hash));
    }

    #[test]
    fn test_multiple_branches_all_shown() {
        let commit = CommitInfo {
            hash: "abc1234".to_string(),
            refs: vec![
                CommitRef {
                    name: "main".to_string(),
                    ref_type: CommitRefType::LocalBranch,
                },
                CommitRef {
                    name: "feature".to_string(),
                    ref_type: CommitRefType::LocalBranch,
                },
                CommitRef {
                    name: "origin/main".to_string(),
                    ref_type: CommitRefType::RemoteBranch,
                },
            ],
            message: "Commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme, None);

        let content = get_span_content(&lines[0]);
        assert!(content.iter().any(|s| s == "main"));
        assert!(content.iter().any(|s| s == "feature"));
        assert!(content.iter().any(|s| s == "origin/main"));
    }

    #[test]
    fn test_checked_out_branch_has_inverted_colors() {
        let commit = CommitInfo {
            hash: "abc1234".to_string(),
            refs: vec![
                CommitRef {
                    name: "main".to_string(),
                    ref_type: CommitRefType::LocalBranch,
                },
                CommitRef {
                    name: "feature".to_string(),
                    ref_type: CommitRefType::LocalBranch,
                },
            ],
            message: "Commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme, Some("main"));

        // Checked out branch has inverted colors (branch color as bg, dark text)
        let main_span = lines[0].spans.iter().find(|s| s.content == "main").unwrap();
        assert_eq!(main_span.style.bg, Some(theme.local_branch));
        assert_eq!(main_span.style.fg, Some(theme.checked_out_branch_bg));

        // Other branches have normal styling
        let feature_span = lines[0]
            .spans
            .iter()
            .find(|s| s.content == "feature")
            .unwrap();
        assert_eq!(feature_span.style.bg, None);
        assert_eq!(feature_span.style.fg, Some(theme.local_branch));
    }
}
