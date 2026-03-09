use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::{
    config::Theme,
    model::{Model, arguments::FetchArgument, popup::FetchPopupState},
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::argument_lines,
    },
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

    let arguments: Vec<Line<'_>> = argument_lines::<FetchArgument>(
        theme,
        model.arg_mode,
        model.arguments.as_ref().and_then(|a| a.fetch()),
    );

    let arguments_col = PopupColumn {
        title: Some("Arguments".into()),
        content: arguments,
    };

    let push_remote_line = {
        let current_branch = model.git_info.current_branch().unwrap_or_default();
        match &state.push_remote {
            Some(remote) => {
                let remote_style = if model.arg_mode {
                    faded_style
                } else {
                    Style::default().fg(theme.remote_branch)
                };
                Line::from(vec![
                    Span::styled(" p", cmd_key_style),
                    Span::styled(" ", cmd_desc_style),
                    Span::styled(format!("{}/{}", remote, current_branch), remote_style),
                ])
            }
            None => Line::from(vec![
                Span::styled(" p", cmd_key_style),
                Span::styled(" ${push-remote}, setting that", cmd_desc_style),
            ]),
        }
    };

    let upstream_line = match &state.upstream {
        Some(upstream) => {
            // Upstream is set - show in remote branch color (or faded if in arg_mode)
            let upstream_style = if model.arg_mode {
                faded_style
            } else {
                Style::default().fg(theme.remote_branch)
            };
            Line::from(vec![
                Span::styled(" u", cmd_key_style),
                Span::styled(" ", cmd_desc_style),
                Span::styled(upstream.clone(), upstream_style),
            ])
        }
        None => {
            // No upstream - show suggestion with ", setting it"
            Line::from(vec![
                Span::styled(" u", cmd_key_style),
                Span::styled(" ${upstream}, setting it", cmd_desc_style),
            ])
        }
    };

    let elsewhere_line = Line::from(vec![
        Span::styled(" e", cmd_key_style),
        Span::styled(" elsewhere", cmd_desc_style),
    ]);

    let all_remotes_line = Line::from(vec![
        Span::styled(" a", cmd_key_style),
        Span::styled(" all remotes", cmd_desc_style),
    ]);

    let fetch_from_col = PopupColumn {
        title: Some("Fetch from".into()),
        content: vec![
            push_remote_line,
            upstream_line,
            elsewhere_line,
            all_remotes_line,
        ],
    };

    let fetch_col = PopupColumn {
        title: Some("Fetch".into()),
        content: vec![
            Line::from(vec![
                Span::styled(" o", cmd_key_style),
                Span::styled(" another branch", cmd_desc_style),
            ]),
            Line::from(vec![
                Span::styled(" r", cmd_key_style),
                Span::styled(" explicit refspec", cmd_desc_style),
            ]),
            Line::from(vec![
                Span::styled(" m", cmd_key_style),
                Span::styled(" submodules", cmd_desc_style),
            ]),
        ],
    };

    CommandPopupContent {
        title: "Fetch",
        rows: vec![
            PopupRow {
                columns: vec![arguments_col],
            },
            PopupRow {
                columns: vec![fetch_from_col],
            },
            PopupRow {
                columns: vec![fetch_col],
            },
        ],
    }
}
