use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::config::Theme;
use crate::model::popup::PushPopupState;

struct ArgumentStyle {
    key_style: Style,
    desc_style: Style,
    dash_style: Style,
    flag_style: Style,
}

pub fn content<'a>(theme: &Theme, state: &PushPopupState) -> CommandPopupContent<'a> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();
    let faded_style = Style::default().fg(Color::DarkGray);

    // Argument key style: '-f' is always green
    let arg_key_style = Style::default()
        .fg(theme.diff_addition)
        .add_modifier(Modifier::BOLD);

    // When in arg_mode, only the '-' prefix is faded
    let arg_dash_style = if state.arg_mode {
        faded_style
    } else {
        arg_key_style
    };

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
        let cmd_key_style = if state.arg_mode {
            faded_style
        } else {
            key_style
        };
        let cmd_desc_style = if state.arg_mode {
            faded_style
        } else {
            desc_style
        };

        let upstream_description = match &state.upstream {
            Some(upstream) => {
                // Upstream is set - show in remote branch color (or faded if in arg_mode)
                let upstream_style = if state.arg_mode {
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

    // Build the arguments section
    // The flag text color depends on whether force_with_lease is selected
    let flag_style = if state.force_with_lease {
        Style::default().fg(theme.diff_addition) // Green when selected
    } else {
        faded_style // Gray when not selected
    };
    let argument_style = ArgumentStyle {
        dash_style: arg_dash_style,
        key_style: arg_key_style,
        desc_style,
        flag_style,
    };

    let arguments: Vec<Line> = vec![argument_line(
        "f",
        "Force with lease",
        "--force-with-lease",
        &argument_style,
    )];

    CommandPopupContent::two_columns("Push", "Commands", commands, "Arguments", arguments)
}

fn argument_line<'a>(
    key: &'a str,
    description: &'a str,
    flag: &'a str,
    style: &ArgumentStyle,
) -> Line<'a> {
    Line::from(vec![
        Span::styled("-", style.dash_style),
        Span::styled(key, style.key_style),
        Span::styled(format!(" {description} ("), style.desc_style),
        Span::styled(flag, style.flag_style),
        Span::styled(")", style.desc_style),
    ])
}
