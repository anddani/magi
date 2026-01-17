use ratatui::{
    style::Style,
    text::{Line as TextLine, Span},
};

use crate::config::Theme;
use crate::git::{CommitInfo, CommitRefType};

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

    // All refs (branches) in order: HEAD/@, current branch, other local, remote
    for commit_ref in &commit.refs {
        spans.push(Span::raw(" "));
        let color = match commit_ref.ref_type {
            CommitRefType::Head => theme.detached_head,
            CommitRefType::LocalBranch => theme.local_branch,
            CommitRefType::RemoteBranch => theme.remote_branch,
        };
        spans.push(Span::styled(
            commit_ref.name.clone(),
            Style::default().fg(color),
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
            tag: None,
            message: "Initial commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme);

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
            tag: None,
            message: "Initial commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme);

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
            ],
            tag: Some("v1.0.0".to_string()),
            message: "Release commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme);

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
            tag: None,
            message: "Detached commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme);

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
            tag: None,
            message: "Detached commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme);

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
            tag: None,
            message: "Commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme);

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
            tag: None,
            message: "Commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme);

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
            tag: None,
            message: "Commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme);

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
            tag: None,
            message: "Commit".to_string(),
        };
        let theme = test_theme();
        let lines = get_lines(&commit, &theme);

        let content = get_span_content(&lines[0]);
        assert!(content.iter().any(|s| s == "main"));
        assert!(content.iter().any(|s| s == "feature"));
        assert!(content.iter().any(|s| s == "origin/main"));
    }
}
