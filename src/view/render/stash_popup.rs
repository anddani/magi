use std::collections::HashSet;

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::{CommandPopupContent, PopupColumn, PopupRow};
use crate::{
    config::Theme,
    model::{
        Model,
        arguments::{Arguments::StashArguments, StashArgument},
    },
    view::render::util::argument_line,
};

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();

    let selected_args: HashSet<StashArgument> =
        if let Some(StashArguments(ref args)) = model.arguments {
            args.clone()
        } else {
            HashSet::new()
        };

    let arguments: Vec<Line> = StashArgument::all()
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

    let stash = PopupColumn {
        title: Some("Stash"),
        content: vec![Line::from(vec![
            Span::styled(" z", key_style),
            Span::styled(" both", desc_style),
        ])],
    };

    CommandPopupContent {
        title: "Stash",
        rows: vec![
            PopupRow {
                columns: vec![arguments_col],
            },
            PopupRow {
                columns: vec![stash],
            },
        ],
    }
}
