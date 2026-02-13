use std::collections::HashSet;

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::{
    config::Theme,
    model::{
        Model,
        arguments::{Arguments::FetchArguments, FetchArgument},
        popup::FetchPopupState,
    },
    view::render::util::{argument_line, column_title},
};

pub fn content<'a>(
    theme: &Theme,
    model: &Model,
    state: &FetchPopupState,
) -> CommandPopupContent<'a> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();
    let faded_style = Style::default().fg(Color::DarkGray);

    // When in arg_mode, fade the command text
    let cmd_key_style = if model.arg_mode {
        faded_style
    } else {
        key_style
    };
    let cmd_desc_style = if model.arg_mode {
        faded_style
    } else {
        desc_style
    };

    let selected_args: HashSet<FetchArgument> =
        if let Some(FetchArguments(ref args)) = model.arguments {
            args.clone()
        } else {
            HashSet::new()
        };

    let mut arguments: Vec<Line> = FetchArgument::all()
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

    let upstream_description = match &state.upstream {
        Some(upstream) => {
            // Upstream is set - show in remote branch color (or faded if in arg_mode)
            let upstream_style = if model.arg_mode {
                faded_style
            } else {
                Style::default().fg(theme.remote_branch)
            };
            vec![
                Span::styled("u", cmd_key_style),
                Span::styled(" ", cmd_desc_style),
                Span::styled(upstream.clone(), upstream_style),
            ]
        }
        None => {
            // No upstream - show suggestion with ", setting it"
            vec![
                Span::styled("u", cmd_key_style),
                Span::styled(" ${upstream}, setting it", cmd_desc_style),
            ]
        }
    };

    let elsewhere_line = Line::from(vec![
        Span::styled("e", cmd_key_style),
        Span::styled(" elsewhere", cmd_desc_style),
    ]);

    let all_remotes_line = Line::from(vec![
        Span::styled("a", cmd_key_style),
        Span::styled(" all remotes", cmd_desc_style),
    ]);

    let mut commands: Vec<Line> = vec![
        Line::from(upstream_description),
        elsewhere_line,
        all_remotes_line,
    ];

    let mut content: Vec<Line> = vec![];
    content.push(column_title("Arguments", theme));
    content.append(&mut arguments);

    content.push(Line::from(""));

    content.push(column_title("Fetch from", theme));
    content.append(&mut commands);

    CommandPopupContent::single_column("Fetch", content)
}
