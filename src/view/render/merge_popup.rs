use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::{CommandPopupContent, PopupColumn, PopupRow};
use crate::{config::Theme, model::popup::MergePopupState};

pub fn content<'a>(theme: &Theme, state: &'a MergePopupState) -> CommandPopupContent<'a> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();

    if state.in_progress {
        // Merge is paused on a conflict — show Continue / Abort
        let key_lines = vec![
            Line::from(vec![
                Span::styled(" m", key_style),
                Span::styled("  Continue", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" a", key_style),
                Span::styled("  Abort", desc_style),
            ]),
        ];
        CommandPopupContent::single_column("Merging", key_lines)
    } else {
        CommandPopupContent {
            title: "Merge",
            rows: vec![PopupRow {
                columns: vec![PopupColumn {
                    title: Some("Actions".into()),
                    content: vec![Line::from(vec![
                        Span::styled(" m", key_style),
                        Span::styled("  Merge", desc_style),
                    ])],
                }],
            }],
        }
    }
}
