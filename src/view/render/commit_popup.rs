use std::collections::HashSet;

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::{
    config::Theme,
    model::{
        Model,
        arguments::{Arguments::CommitArguments, CommitArgument},
    },
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::argument_line,
    },
};

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();

    let selected_args: HashSet<CommitArgument> =
        if let Some(CommitArguments(ref args)) = model.arguments {
            args.clone()
        } else {
            HashSet::new()
        };

    let arguments: Vec<Line> = CommitArgument::all()
        .iter()
        .map(|arg| {
            argument_line(
                theme,
                arg.key(),
                arg.description(),
                arg.flag(),
                model.arg_mode,
                selected_args.contains(arg),
            )
        })
        .collect();

    let arguments_col = PopupColumn {
        title: Some("Arguments"),
        content: arguments,
    };

    let create_col = PopupColumn {
        title: Some("Create"),
        content: vec![Line::from(vec![
            Span::styled(" c", key_style),
            Span::styled(" Commit", desc_style),
        ])],
    };

    let edit_head_col = PopupColumn {
        title: Some("Edit HEAD"),
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
        title: Some("Edit"),
        content: vec![
            Line::from(vec![
                Span::styled(" f", key_style),
                Span::styled(" Fixup", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" s", key_style),
                Span::styled(" Squash", desc_style),
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
