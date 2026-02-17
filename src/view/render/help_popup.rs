use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::config::Theme;

pub fn content(theme: &Theme) -> CommandPopupContent<'static> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();
    let section_style = Style::default()
        .fg(theme.section_header)
        .add_modifier(Modifier::BOLD);

    let commands: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("  b       ", key_style),
            Span::styled("Branch", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  c       ", key_style),
            Span::styled("Commit", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  f       ", key_style),
            Span::styled("Fetch", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  F       ", key_style),
            Span::styled("Pull", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  l       ", key_style),
            Span::styled("Log", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  P       ", key_style),
            Span::styled("Push", desc_style),
        ]),
        Line::from(""),
        // General section
        Line::from(Span::styled("Applying changes", section_style)),
        Line::from(vec![
            Span::styled("  s       ", key_style),
            Span::styled("Stage", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  S       ", key_style),
            Span::styled("Stage all", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  u       ", key_style),
            Span::styled("Unstage", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  U       ", key_style),
            Span::styled("Unstage all", desc_style),
        ]),
    ];

    let general: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("  q       ", key_style),
            Span::styled("Quit", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl-r  ", key_style),
            Span::styled("Refresh", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  ?       ", key_style),
            Span::styled("Show this help", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  j/Down  ", key_style),
            Span::styled("Move down", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  k/Up    ", key_style),
            Span::styled("Move up", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl-d  ", key_style),
            Span::styled("Half page down", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl-u  ", key_style),
            Span::styled("Half page up", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  gg      ", key_style),
            Span::styled("Go to first line", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  G       ", key_style),
            Span::styled("Go to last line", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl-e  ", key_style),
            Span::styled("Scroll line down", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl-y  ", key_style),
            Span::styled("Scroll line up", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Tab     ", key_style),
            Span::styled("Toggle section collapse/expand", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  V       ", key_style),
            Span::styled("Enter visual selection mode", desc_style),
        ]),
    ];

    CommandPopupContent::two_columns("Help", "Commands", commands, "General", general)
}
