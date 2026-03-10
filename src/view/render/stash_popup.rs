use ratatui::text::Line;

use super::popup_content::{CommandPopupContent, PopupColumn, PopupRow};

use crate::{
    config::Theme,
    i18n,
    model::{Model, arguments::StashArgument},
    view::render::util::{argument_lines, command_description},
};

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let t = i18n::t();

    let arguments: Vec<Line<'_>> = argument_lines::<StashArgument>(
        theme,
        model.arg_mode,
        model.arguments.as_ref().and_then(|a| a.stash()),
    );

    let arguments_col = PopupColumn {
        title: Some(t.col_arguments.into()),
        content: arguments,
    };

    let stash = PopupColumn {
        title: Some(t.popup_stash.into()),
        content: vec![
            command_description(theme, model.arg_mode, "z", t.cmd_both),
            command_description(theme, model.arg_mode, "i", t.cmd_index),
            command_description(theme, model.arg_mode, "w", t.cmd_worktree),
        ],
    };

    let use_col = PopupColumn {
        title: Some(t.col_use.into()),
        content: vec![
            command_description(theme, model.arg_mode, "a", t.cmd_apply),
            command_description(theme, model.arg_mode, "p", t.cmd_pop),
            command_description(theme, model.arg_mode, "k", t.cmd_drop),
        ],
    };

    CommandPopupContent {
        title: t.popup_stash,
        rows: vec![
            PopupRow {
                columns: vec![arguments_col],
            },
            PopupRow {
                columns: vec![stash, use_col],
            },
        ],
    }
}
