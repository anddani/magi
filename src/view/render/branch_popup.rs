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

    let checkout = PopupColumn {
        title: Some("Checkout"),
        content: vec![
            Line::from(vec![
                Span::styled(" b", key_style),
                Span::styled(" branch/revision", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" l", key_style),
                Span::styled(" local branch", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" c", key_style),
                Span::styled(" new branch", desc_style),
            ]),
        ],
    };

    let create = PopupColumn {
        title: Some("Create"),
        content: vec![
            Line::from(vec![
                Span::styled(" n", key_style),
                Span::styled(" new branch", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" o", key_style),
                Span::styled(" new PR to default branch", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" O", key_style),
                Span::styled(" new PR to...", desc_style),
            ]),
        ],
    };

    let do_col = PopupColumn {
        title: Some("Do"),
        content: vec![
            Line::from(vec![
                Span::styled(" m", key_style),
                Span::styled(" rename", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" x", key_style),
                Span::styled(" delete", desc_style),
            ]),
        ],
    };

    CommandPopupContent {
        title: "Branch",
        rows: vec![PopupRow {
            columns: vec![checkout, create, do_col],
        }],
    }
}
