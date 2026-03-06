use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::{
    config::Theme,
    view::render::popup_content::{PopupColumn, PopupRow},
};

pub fn content<'a>(theme: &Theme) -> CommandPopupContent<'a> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();

    let reset_col = PopupColumn {
        title: Some("Reset"),
        content: vec![
            Line::from(vec![
                Span::styled(" b", key_style),
                Span::styled(" branch", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" f", key_style),
                Span::styled(" file", desc_style),
            ]),
        ],
    };

    let reset_this_col = PopupColumn {
        title: Some("Reset this"),
        content: vec![
            Line::from(vec![
                Span::styled(" m", key_style),
                Span::styled(" mixed    (HEAD and index)", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" s", key_style),
                Span::styled(" soft     (HEAD only)", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" h", key_style),
                Span::styled(" hard     (HEAD, index and worktree)", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" k", key_style),
                Span::styled(
                    " keep     (HEAD and index, keeping uncommitted)",
                    desc_style,
                ),
            ]),
            Line::from(vec![
                Span::styled(" i", key_style),
                Span::styled(" index    (only)", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" w", key_style),
                Span::styled(" worktree (only)", desc_style),
            ]),
        ],
    };

    CommandPopupContent {
        title: "Reset",
        rows: vec![PopupRow {
            columns: vec![reset_col, reset_this_col],
        }],
    }
}
