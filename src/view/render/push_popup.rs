use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::{
    config::Theme,
    i18n,
    model::{Model, arguments::PushArgument, popup::PushPopupState},
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
    state: &PushPopupState,
) -> CommandPopupContent<'a> {
    let t = i18n::t();

    let push_to_title = match model.git_info.current_branch() {
        Some(branch) => {
            let column_title_style = Style::default()
                .fg(theme.section_header)
                .add_modifier(Modifier::BOLD);
            let branch_style = Style::default()
                .fg(theme.local_branch)
                .add_modifier(Modifier::BOLD);
            Line::from(vec![
                Span::styled(t.push_to_pre, column_title_style),
                Span::styled(branch, branch_style),
                Span::styled(t.push_to_post, column_title_style),
            ])
        }
        None => column_title(t.push_to_fallback, theme),
    };

    let arguments: Vec<Line<'_>> = argument_lines::<PushArgument>(
        theme,
        model.arg_mode,
        model.arguments.as_ref().and_then(|a| a.push()),
    );

    let arguments_col = PopupColumn {
        title: Some(t.col_arguments.into()),
        content: arguments,
    };

    let push_to_col = PopupColumn {
        title: Some(PopupColumnTitle::Styled(push_to_title)),
        content: vec![
            push_remote_description(model, theme, &state.push_remote),
            upstream_description(theme, model.arg_mode, &state.upstream),
            command_description(theme, model.arg_mode, "e", t.cmd_elsewhere),
        ],
    };

    let push_1_col = PopupColumn {
        title: Some(PopupColumnTitle::Raw(t.popup_push)),
        content: vec![
            command_description(theme, model.arg_mode, "o", t.cmd_other_branch),
            command_description(theme, model.arg_mode, "r", t.cmd_explicit_refspec),
            command_description(theme, model.arg_mode, "m", t.cmd_matching_branches),
        ],
    };

    let push_2_col = PopupColumn {
        title: None,
        content: vec![
            Line::from(""),
            command_description(theme, model.arg_mode, "T", t.cmd_push_tag),
            command_description(theme, model.arg_mode, "t", t.cmd_push_all_tags),
        ],
    };

    CommandPopupContent {
        title: t.popup_push,
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
