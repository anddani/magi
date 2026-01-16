use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::config::Theme;

pub fn content(theme: &Theme) -> (&str, Vec<Line<'_>>) {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();
    let section_style = Style::default()
        .fg(theme.section_header)
        .add_modifier(Modifier::BOLD);
    let hint_style = Style::default().fg(Color::DarkGray);

    let title = "Help";
    let body: Vec<Line> = vec![
        // Navigation section
        Line::from(Span::styled("Navigation", section_style)),
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
        Line::from(""),
        // Actions section
        Line::from(Span::styled("Actions", section_style)),
        Line::from(vec![
            Span::styled("  c       ", key_style),
            Span::styled("Commit", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  S       ", key_style),
            Span::styled("Stage all modified files", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  U       ", key_style),
            Span::styled("Unstage all files", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  V       ", key_style),
            Span::styled("Enter visual selection mode", desc_style),
        ]),
        Line::from(""),
        // General section
        Line::from(Span::styled("General", section_style)),
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
        Line::from(""),
        Line::from(Span::styled("Press Enter or Esc to dismiss", hint_style)),
    ];
    (title, body)
}
