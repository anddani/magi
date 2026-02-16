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

    let commands: Vec<Line> = vec![Line::from(vec![
        Span::styled("l", key_style),
        Span::styled(" current", desc_style),
    ])];

    CommandPopupContent::single_column("Log", commands)
}
