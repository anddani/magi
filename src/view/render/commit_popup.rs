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
    view::render::util::{argument_line, column_title},
};

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();

    let mut content: Vec<Line> = vec![];

    let selected_args: HashSet<CommitArgument> =
        if let Some(CommitArguments(ref args)) = model.arguments {
            args.clone()
        } else {
            HashSet::new()
        };

    let mut arguments: Vec<Line> = CommitArgument::all()
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
    content.push(column_title("Arguments", theme));
    content.append(&mut arguments);

    content.push(Line::from(""));

    content.push(column_title("Create", theme));
    content.push(Line::from(vec![
        Span::styled("c", key_style),
        Span::styled(" Commit", desc_style),
    ]));

    content.push(Line::from(""));

    content.push(column_title("Edit HEAD", theme));
    content.append(&mut vec![
        Line::from(vec![
            Span::styled("e", key_style),
            Span::styled(" Extend", desc_style),
        ]),
        Line::from(vec![
            Span::styled("a", key_style),
            Span::styled(" Amend", desc_style),
        ]),
        Line::from(vec![
            Span::styled("w", key_style),
            Span::styled(" Reword", desc_style),
        ]),
        Line::from(vec![
            Span::styled("f", key_style),
            Span::styled(" Fixup", desc_style),
        ]),
        Line::from(vec![
            Span::styled("s", key_style),
            Span::styled(" Squash", desc_style),
        ]),
    ]);

    CommandPopupContent::single_column("Commit", content)
}
