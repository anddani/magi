use ratatui::text::Line;

use super::popup_content::CommandPopupContent;
use crate::{
    config::Theme,
    i18n,
    model::{Model, arguments::CommitArgument},
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::{argument_lines, command_description},
    },
};

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let t = i18n::t();
    let arguments: Vec<Line<'_>> = argument_lines::<CommitArgument>(
        theme,
        model.arg_mode,
        model.arguments.as_ref().and_then(|a| a.commit()),
    );

    let arguments_col = PopupColumn {
        title: Some(t.col_arguments.into()),
        content: arguments,
    };

    let create_col = PopupColumn {
        title: Some(t.col_create.into()),
        content: vec![command_description(
            theme,
            model.arg_mode,
            "c",
            t.cmd_commit,
        )],
    };

    let edit_head_col = PopupColumn {
        title: Some(t.col_edit_head.into()),
        content: vec![
            command_description(theme, model.arg_mode, "e", t.cmd_extend),
            Line::from(""),
            command_description(theme, model.arg_mode, "a", t.cmd_amend),
            Line::from(""),
            command_description(theme, model.arg_mode, "w", t.cmd_reword),
        ],
    };

    let edit_col = PopupColumn {
        title: Some(t.col_edit.into()),
        content: vec![
            command_description(theme, model.arg_mode, "f", t.cmd_fixup),
            command_description(theme, model.arg_mode, "s", t.cmd_squash),
            command_description(theme, model.arg_mode, "A", t.cmd_alter),
            command_description(theme, model.arg_mode, "n", t.cmd_augment),
            command_description(theme, model.arg_mode, "W", t.cmd_revise),
        ],
    };

    CommandPopupContent {
        title: t.popup_commit,
        rows: vec![
            PopupRow {
                columns: vec![arguments_col],
            },
            PopupRow {
                columns: vec![create_col, edit_head_col, edit_col],
            },
        ],
    }
}
