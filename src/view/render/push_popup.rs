use std::collections::HashSet;

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::model::arguments::{Arguments::PushArguments, PushArgument};
use crate::model::popup::PushPopupState;
use crate::{config::Theme, model::Model, view::render::util::argument_line};

pub fn content<'a>(
    theme: &Theme,
    model: &Model,
    state: &PushPopupState,
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

    let push_tags = vec![
        Span::styled("t", cmd_key_style),
        Span::styled(" Push all tags", cmd_desc_style),
    ];

    let push_single_tag = vec![
        Span::styled("T", cmd_key_style),
        Span::styled(" Push a tag", cmd_desc_style),
    ];

    let commands: Vec<Line> = vec![
        Line::from(upstream_description),
        Line::from(push_tags),
        Line::from(push_single_tag),
    ];

    let selected_args: HashSet<PushArgument> =
        if let Some(PushArguments(ref args)) = model.arguments {
            args.clone()
        } else {
            HashSet::new()
        };

    let arguments: Vec<Line> = PushArgument::all()
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

    CommandPopupContent::two_columns("Push", "Commands", commands, "Arguments", arguments)
}
