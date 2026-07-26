use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::view::util::ref_style;
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

    // Graph portion - colored by git (--color) or with the theme color
    if !entry.graph.is_empty() {
        spans.extend(graph_spans(
            &entry.graph,
            Style::default().fg(theme.diff_context),
        ));
    }

    // Hash - colored by signature status when the log was requested with
    // --show-signature, like magit's magit-signature-* faces
    if let Some(ref hash) = entry.hash {
        spans.push(Span::styled(
            hash.clone(),
            Style::default()
                .fg(signature_color(entry.signature, theme))
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
            let is_current = current_branch == Some(commit_ref.name.as_str());

            if commit_ref.ref_type == CommitRefType::LocalBranch
                && let Some(remote) = &commit_ref.push_remote
            {
                // Split-colored label: "remote/" in remote_branch color + "branch" in local_branch color
                spans.push(Span::styled(
                    format!("{}/", remote),
                    ref_style(theme.remote_branch, is_current),
                ));
                spans.push(Span::styled(
                    commit_ref.name.clone(),
                    ref_style(theme.local_branch, is_current),
                ));
                spans.push(Span::raw(" "));
                continue;
            }

            let color = match commit_ref.ref_type {
                CommitRefType::Head => theme.detached_head,
                CommitRefType::LocalBranch => theme.local_branch,
                CommitRefType::RemoteBranch => theme.remote_branch,
                CommitRefType::Tag => theme.tag_label,
            };
            spans.push(Span::styled(
                commit_ref.name.clone(),
                ref_style(color, is_current),
            ));
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
            Style::default().fg(theme.dim_text),
        ));
    }

    vec![Line::from(spans)]
}

/// Color for a commit hash based on its `%G?` signature status:
/// good signatures are green, broken ones (bad, revoked, error) red and
/// questionable ones (untrusted, expired) yellow. Unsigned commits (N)
/// and logs without --show-signature use the normal hash color.
fn signature_color(signature: Option<char>, theme: &Theme) -> Color {
    match signature {
        Some('G') => theme.diff_addition,
        Some('B') | Some('R') | Some('E') => theme.diff_deletion,
        Some('U') | Some('X') | Some('Y') => theme.section_header,
        _ => theme.commit_hash,
    }
}

/// Split a graph string on the ANSI color codes emitted by `git log --color`
/// into styled spans. Text outside any color code uses `default_style`.
fn graph_spans(graph: &str, default_style: Style) -> Vec<Span<'static>> {
    let mut spans: Vec<Span> = Vec::new();
    let mut style = default_style;
    let mut text = String::new();
    let mut rest = graph;

    while let Some(esc) = rest.find('\x1b') {
        text.push_str(&rest[..esc]);
        rest = &rest[esc..];

        let Some(end) = rest.find('m') else { break };
        let new_style = apply_sgr(&rest[..end + 1], style, default_style);
        rest = &rest[end + 1..];

        if new_style != style {
            if !text.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut text), style));
            }
            style = new_style;
        }
    }

    text.push_str(rest);
    if !text.is_empty() {
        spans.push(Span::styled(text, style));
    }

    spans
}

/// Apply an SGR escape sequence (e.g. "\x1b[1;31m") on top of `style`.
/// Unsupported parameters are ignored; a reset restores `default_style`.
fn apply_sgr(sequence: &str, style: Style, default_style: Style) -> Style {
    let Some(params) = sequence
        .strip_prefix("\x1b[")
        .and_then(|s| s.strip_suffix('m'))
    else {
        return style;
    };

    let mut style = style;
    let mut params = params.split(';').map(|p| p.parse::<u8>().unwrap_or(0));
    while let Some(param) = params.next() {
        match param {
            0 => style = default_style,
            1 => style = style.add_modifier(Modifier::BOLD),
            2 => style = style.add_modifier(Modifier::DIM),
            30..=37 => style = style.fg(ansi_color(param - 30, false)),
            90..=97 => style = style.fg(ansi_color(param - 90, true)),
            38 => match (params.next(), params.next()) {
                (Some(5), Some(idx)) => style = style.fg(Color::Indexed(idx)),
                (Some(2), Some(r)) => {
                    let (g, b) = (params.next().unwrap_or(0), params.next().unwrap_or(0));
                    style = style.fg(Color::Rgb(r, g, b));
                }
                _ => {}
            },
            _ => {}
        }
    }
    style
}

/// Map an ANSI color number (0-7) to a ratatui color
fn ansi_color(number: u8, bright: bool) -> Color {
    match (number, bright) {
        (0, false) => Color::Black,
        (1, false) => Color::Red,
        (2, false) => Color::Green,
        (3, false) => Color::Yellow,
        (4, false) => Color::Blue,
        (5, false) => Color::Magenta,
        (6, false) => Color::Cyan,
        (_, false) => Color::Gray,
        (0, true) => Color::DarkGray,
        (1, true) => Color::LightRed,
        (2, true) => Color::LightGreen,
        (3, true) => Color::LightYellow,
        (4, true) => Color::LightBlue,
        (5, true) => Color::LightMagenta,
        (6, true) => Color::LightCyan,
        (_, true) => Color::White,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_style() -> Style {
        Style::default().fg(Color::Gray)
    }

    #[test]
    fn test_graph_spans_plain() {
        let spans = graph_spans("| * ", default_style());
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content, "| * ");
        assert_eq!(spans[0].style, default_style());
    }

    #[test]
    fn test_graph_spans_colored() {
        let spans = graph_spans("\x1b[31m|\x1b[m \x1b[1;32m*\x1b[m ", default_style());
        assert_eq!(spans.len(), 4);
        assert_eq!(spans[0].content, "|");
        assert_eq!(spans[0].style, default_style().fg(Color::Red));
        assert_eq!(spans[1].content, " ");
        assert_eq!(spans[1].style, default_style());
        assert_eq!(spans[2].content, "*");
        assert_eq!(
            spans[2].style,
            default_style()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        );
        assert_eq!(spans[3].content, " ");
        assert_eq!(spans[3].style, default_style());
    }

    #[test]
    fn test_graph_spans_256_and_rgb_colors() {
        let spans = graph_spans(
            "\x1b[38;5;208m|\x1b[m\x1b[38;2;1;2;3m/\x1b[m",
            default_style(),
        );
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].style, default_style().fg(Color::Indexed(208)));
        assert_eq!(spans[1].style, default_style().fg(Color::Rgb(1, 2, 3)));
    }

    #[test]
    fn test_graph_spans_unsupported_codes_ignored() {
        let spans = graph_spans("\x1b[4m|\x1b[m *", default_style());
        let text: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "| *");
    }

    #[test]
    fn test_signature_color() {
        let theme = crate::config::Theme::default();
        assert_eq!(signature_color(Some('G'), &theme), theme.diff_addition);
        assert_eq!(signature_color(Some('B'), &theme), theme.diff_deletion);
        assert_eq!(signature_color(Some('R'), &theme), theme.diff_deletion);
        assert_eq!(signature_color(Some('E'), &theme), theme.diff_deletion);
        assert_eq!(signature_color(Some('U'), &theme), theme.section_header);
        assert_eq!(signature_color(Some('X'), &theme), theme.section_header);
        assert_eq!(signature_color(Some('Y'), &theme), theme.section_header);
        // Unsigned commits and logs without --show-signature keep the hash color
        assert_eq!(signature_color(Some('N'), &theme), theme.commit_hash);
        assert_eq!(signature_color(None, &theme), theme.commit_hash);
    }
}
