use std::collections::HashSet;

use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::{
    config::Theme,
    model::{
        arguments::{Arguments::FetchArguments, FetchArgument},
        Model,
    },
    view::render::util::argument_line,
};

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();

    let commands: Vec<Line> = vec![Line::from(vec![
        Span::styled("a", key_style),
        Span::styled(" all remotes", desc_style),
    ])];

    let selected_args: HashSet<FetchArgument> =
        if let Some(FetchArguments(ref args)) = model.arguments {
            args.clone()
        } else {
            HashSet::new()
        };

    let arguments: Vec<Line> = FetchArgument::all()
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

    CommandPopupContent::two_columns("Fetch", "Fetch from", commands, "Arguments", arguments)
}
