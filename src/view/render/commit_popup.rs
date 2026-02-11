use std::collections::HashSet;

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::{
    config::Theme,
    model::{Model, arguments::Arguments::CommitArguments, arguments::CommitArgument},
    view::render::util::argument_line,
};

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();

    let commands: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("c", key_style),
            Span::styled(" Commit", desc_style),
        ]),
        Line::from(vec![
            Span::styled("a", key_style),
            Span::styled(" Amend", desc_style),
        ]),
    ];

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

    CommandPopupContent::two_columns("Commit", "Commands", commands, "Arguments", arguments)
}
