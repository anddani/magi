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

    let commands: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("c", key_style),
            Span::styled(" Commit", desc_style),
        ]),
        Line::from(vec![
            Span::styled("a", key_style),
            Span::styled(" Amend", desc_style),
        ]),
    ];

    let arguments: Vec<Line> = vec![];

    CommandPopupContent::two_columns("Commit", "Commands", commands, "Arguments", arguments)
}
