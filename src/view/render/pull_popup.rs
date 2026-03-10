use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::{
    config::Theme,
    i18n,
    model::{Model, arguments::PullArgument, popup::PullPopupState},
    view::render::{
        popup_content::{PopupColumn, PopupColumnTitle, PopupRow},
        util::{
            argument_lines, column_title, command_description, push_remote_description,
            upstream_description,
        },
    },
};

pub fn content<'a>(
    theme: &Theme,
    model: &Model,
    state: &PullPopupState,
) -> CommandPopupContent<'a> {
    let t = i18n::t();

    let arguments: Vec<Line<'_>> = argument_lines::<PullArgument>(
        theme,
        model.arg_mode,
        model.arguments.as_ref().and_then(|a| a.pull()),
    );

    let arguments_col = PopupColumn {
        title: Some(t.col_arguments.into()),
        content: arguments,
    };

    let pull_into_title = match model.git_info.current_branch() {
        Some(branch) => {
            let column_title_style = Style::default()
                .fg(theme.section_header)
                .add_modifier(Modifier::BOLD);
            let branch_style = Style::default()
                .fg(theme.local_branch)
                .add_modifier(Modifier::BOLD);
            Line::from(vec![
                Span::styled(t.pull_into_pre, column_title_style),
                Span::styled(branch, branch_style),
                Span::styled(t.pull_into_post, column_title_style),
            ])
        }
        None => column_title(t.pull_into_fallback, theme),
    };

    let pull_into_col = PopupColumn {
        title: Some(PopupColumnTitle::Styled(pull_into_title)),
        content: vec![
            push_remote_description(model, theme, &state.push_remote),
            upstream_description(theme, model.arg_mode, &state.upstream),
            command_description(theme, model.arg_mode, "e", t.cmd_elsewhere),
        ],
    };

    CommandPopupContent {
        title: t.popup_pull,
        rows: vec![
            PopupRow {
                columns: vec![arguments_col],
            },
            PopupRow {
                columns: vec![pull_into_col],
            },
        ],
    }
}
