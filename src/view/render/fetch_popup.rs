use ratatui::text::Line;

use super::popup_content::CommandPopupContent;
use crate::{
    config::Theme,
    i18n,
    model::{Model, arguments::FetchArgument, popup::FetchPopupState},
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::{
            argument_lines, command_description, push_remote_description, upstream_description,
        },
    },
};

pub fn content<'a>(
    theme: &Theme,
    model: &Model,
    state: &FetchPopupState,
) -> CommandPopupContent<'a> {
    let t = i18n::t();

    let arguments: Vec<Line<'_>> = argument_lines::<FetchArgument>(
        theme,
        model.arg_mode,
        model.arguments.as_ref().and_then(|a| a.fetch()),
    );

    let arguments_col = PopupColumn {
        title: Some(t.col_arguments.into()),
        content: arguments,
    };

    let fetch_from_col = PopupColumn {
        title: Some(t.col_fetch_from.into()),
        content: vec![
            push_remote_description(model, theme, &state.push_remote),
            upstream_description(theme, model.arg_mode, &state.upstream),
            command_description(theme, model.arg_mode, "e", t.cmd_elsewhere),
            command_description(theme, model.arg_mode, "a", t.cmd_all_remotes),
        ],
    };

    let fetch_col = PopupColumn {
        title: Some(t.popup_fetch.into()),
        content: vec![
            command_description(theme, model.arg_mode, "o", t.cmd_another_branch),
            command_description(theme, model.arg_mode, "r", t.cmd_explicit_refspec),
            command_description(theme, model.arg_mode, "m", t.cmd_submodules),
        ],
    };

    CommandPopupContent {
        title: t.popup_fetch,
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
