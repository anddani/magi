use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::{CommandPopupContent, PopupColumn, PopupRow};
use crate::{
    config::Theme,
    model::{Model, arguments::StashArgument},
    view::render::util::argument_lines,
};

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();

    let arguments: Vec<Line<'_>> = argument_lines::<StashArgument>(
        theme,
        model.arg_mode,
        model.arguments.as_ref().and_then(|a| a.stash()),
    );

    let arguments_col = PopupColumn {
        title: Some("Arguments".into()),
        content: arguments,
    };

    let stash = PopupColumn {
        title: Some("Stash".into()),
        content: vec![
            Line::from(vec![
                Span::styled(" z", key_style),
                Span::styled(" both", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" i", key_style),
                Span::styled(" index", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" w", key_style),
                Span::styled(" worktree", desc_style),
            ]),
        ],
    };

    let use_col = PopupColumn {
        title: Some("Use".into()),
        content: vec![
            Line::from(vec![
                Span::styled(" a", key_style),
                Span::styled(" Apply", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" p", key_style),
                Span::styled(" Pop", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" k", key_style),
                Span::styled(" Drop", desc_style),
            ]),
        ],
    };

    CommandPopupContent {
        title: "Stash",
        rows: vec![
            PopupRow {
                columns: vec![arguments_col],
            },
            PopupRow {
                columns: vec![stash, use_col],
            },
        ],
    }
}
