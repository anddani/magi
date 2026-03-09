use std::collections::HashSet;

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::{
    config::Theme,
    model::Model,
    view::render::{popup_content::PopupColumnTitle, util::argument_line},
};
use crate::{
    model::arguments::{Arguments::PushArguments, PushArgument},
    view::render::util::column_title,
};
use crate::{
    model::popup::PushPopupState,
    view::render::popup_content::{PopupColumn, PopupRow},
};

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

    let push_to_title = match model.git_info.current_branch() {
        Some(branch) => {
            let column_title_style = Style::default()
                .fg(theme.section_header)
                .add_modifier(Modifier::BOLD);
            let branch_style = Style::default()
                .fg(theme.local_branch)
                .add_modifier(Modifier::BOLD);
            Line::from(vec![
                Span::styled("Push ", column_title_style),
                Span::styled(branch, branch_style),
                Span::styled(" to", column_title_style),
            ])
        }
        None => column_title("Push to", theme),
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

    let arguments_col = PopupColumn {
        title: Some("Arguments".into()),
        content: arguments,
    };

    let push_remote_description = {
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

    let upstream_description = match &state.upstream {
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

    let push_to_col = PopupColumn {
        title: Some(PopupColumnTitle::Styled(push_to_title)),
        content: vec![
            push_remote_description,
            upstream_description,
            Line::from(vec![
                Span::styled(" e", cmd_key_style),
                Span::styled(" elsewhere", cmd_desc_style),
            ]),
        ],
    };

    let push_1_col = PopupColumn {
        title: Some(PopupColumnTitle::Raw("Push")),
        content: vec![
            Line::from(vec![
                Span::styled(" o", cmd_key_style),
                Span::styled(" other branch", cmd_desc_style),
            ]),
            Line::from(vec![
                Span::styled(" r", cmd_key_style),
                Span::styled(" explicit refspec", cmd_desc_style),
            ]),
            Line::from(vec![
                Span::styled(" m", cmd_key_style),
                Span::styled(" matching branches", cmd_desc_style),
            ]),
        ],
    };

    let push_2_col = PopupColumn {
        title: None,
        content: vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(" T", cmd_key_style),
                Span::styled(" Push a tag", cmd_desc_style),
            ]),
            Line::from(vec![
                Span::styled(" t", cmd_key_style),
                Span::styled(" Push all tags", cmd_desc_style),
            ]),
        ],
    };

    CommandPopupContent {
        title: "Reset",
        rows: vec![
            PopupRow {
                columns: vec![arguments_col],
            },
            PopupRow {
                columns: vec![push_to_col],
            },
            PopupRow {
                columns: vec![push_1_col, push_2_col],
            },
        ],
    }
}
