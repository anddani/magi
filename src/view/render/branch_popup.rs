use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::{config::Theme, view::render::util::column_title};

pub fn content(theme: &Theme) -> CommandPopupContent<'static> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();

    let content: Vec<Line> = vec![
        column_title("Checkout", theme),
        Line::from(vec![
            Span::styled("b", key_style),
            Span::styled(" branch/revision", desc_style),
        ]),
        Line::from(vec![
            Span::styled("l", key_style),
            Span::styled(" local branch", desc_style),
        ]),
        Line::from(vec![
            Span::styled("c", key_style),
            Span::styled(" new branch", desc_style),
        ]),
        Line::from(vec![
            Span::styled("x", key_style),
            Span::styled(" branch", desc_style),
        ]),
        Line::from(""),
        column_title("Create", theme),
        Line::from(vec![
            Span::styled("o", key_style),
            Span::styled(" new PR to default branch", desc_style),
        ]),
        Line::from(vec![
            Span::styled("O", key_style),
            Span::styled(" new PR to...", desc_style),
        ]),
    ];

    CommandPopupContent::single_column("Branch", content)
}
