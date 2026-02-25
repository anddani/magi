use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::{CommandPopupContent, PopupColumn, PopupRow};
use crate::config::Theme;

pub fn content(theme: &Theme) -> CommandPopupContent<'static> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();

    let stash = PopupColumn {
        title: Some("Stash"),
        content: vec![Line::from(vec![
            Span::styled(" z", key_style),
            Span::styled(" both", desc_style),
        ])],
    };

    CommandPopupContent {
        title: "Stash",
        rows: vec![PopupRow {
            columns: vec![stash],
        }],
    }
}
