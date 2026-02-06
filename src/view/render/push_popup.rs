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

    let commands: Vec<Line> = if state.input_mode {
        // In input mode, show the input field
        let suggested = format!("{}/{}", state.default_remote, state.local_branch);
        let input_display = if state.input_text.is_empty() {
            // Show placeholder (remote/branch) in faded text
            vec![
                Span::styled("u", key_style),
                Span::styled(" ", desc_style),
                Span::styled(suggested, faded_style),
                Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
            ]
        } else {
            // Show entered text with cursor
            vec![
                Span::styled("u", key_style),
                Span::styled(" ", desc_style),
                Span::styled(state.input_text.clone(), desc_style),
                Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
            ]
        };
        vec![
            Line::from(input_display),
            Line::from(vec![Span::styled(
                "  Tab to complete, Enter to confirm, Esc to cancel",
                faded_style,
            )]),
        ]
    } else {
        // Normal mode - show commands
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
                // No upstream - show suggestion with ", creating it"
                vec![
                    Span::styled("u", cmd_key_style),
                    Span::styled(" ${upstream}, creating it", cmd_desc_style),
                ]
            }
        };
        vec![Line::from(upstream_description)]
    };

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
