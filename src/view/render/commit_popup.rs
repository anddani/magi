use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::{
    config::Theme,
    model::{Model, arguments::CommitArgument},
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::argument_lines,
    },
};

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();

    let arguments: Vec<Line<'_>> = argument_lines::<CommitArgument>(
        theme,
        model.arg_mode,
        model.arguments.as_ref().and_then(|a| a.commit()),
    );

    let arguments_col = PopupColumn {
        title: Some("Arguments".into()),
        content: arguments,
    };

    let create_col = PopupColumn {
        title: Some("Create".into()),
        content: vec![Line::from(vec![
            Span::styled(" c", key_style),
            Span::styled(" Commit", desc_style),
        ])],
    };

    let edit_head_col = PopupColumn {
        title: Some("Edit HEAD".into()),
        content: vec![
            Line::from(vec![
                Span::styled(" e", key_style),
                Span::styled(" Extend", desc_style),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" a", key_style),
                Span::styled(" Amend", desc_style),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" w", key_style),
                Span::styled(" Reword", desc_style),
            ]),
        ],
    };

    let edit_col = PopupColumn {
        title: Some("Edit".into()),
        content: vec![
            Line::from(vec![
                Span::styled(" f", key_style),
                Span::styled(" Fixup", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" s", key_style),
                Span::styled(" Squash", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" A", key_style),
                Span::styled(" Alter", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" n", key_style),
                Span::styled(" Augment", desc_style),
            ]),
        ],
    };

    CommandPopupContent {
        title: "Commit",
        rows: vec![
            PopupRow {
                columns: vec![arguments_col],
            },
            PopupRow {
                columns: vec![create_col, edit_head_col, edit_col],
            },
        ],
    }
}
