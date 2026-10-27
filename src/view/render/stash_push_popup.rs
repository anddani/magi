use ratatui::text::Line;

use super::popup_content::{CommandPopupContent, PopupColumn, PopupRow};

use crate::{
    config::Theme,
    i18n,
    model::{Model, arguments::StashPushArgument},
    view::render::util::{argument_lines, command_description},
};

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let t = i18n::t();

    let arguments: Vec<Line<'_>> = argument_lines::<StashPushArgument>(
        theme,
        model.arg_mode,
        model.arguments.as_ref().and_then(|a| a.stash_push()),
    );

    let arguments_col = PopupColumn {
        title: Some(t.col_arguments.into()),
        content: arguments,
    };

    let actions = PopupColumn {
        title: Some(t.col_actions.into()),
        content: vec![command_description(
            theme,
            model.arg_mode,
            "P",
            t.cmd_stash_push,
        )],
    };

    CommandPopupContent {
        title: t.popup_stash_push,
        rows: vec![
            PopupRow {
                columns: vec![arguments_col],
            },
            PopupRow {
                columns: vec![actions],
            },
        ],
    }
}
