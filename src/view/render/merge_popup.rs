use super::popup_content::{CommandPopupContent, PopupColumn, PopupRow};

use crate::{
    config::Theme,
    i18n,
    model::{Model, popup::MergePopupState},
    view::render::util::command_description,
};

pub fn content<'a>(
    theme: &Theme,
    model: &Model,
    state: &'a MergePopupState,
) -> CommandPopupContent<'a> {
    let t = i18n::t();

    if state.in_progress {
        // Merge is paused on a conflict — show Continue / Abort
        return CommandPopupContent {
            title: t.popup_merging,
            rows: vec![PopupRow {
                columns: vec![PopupColumn {
                    title: None,
                    content: vec![
                        command_description(theme, model.arg_mode, "m", t.cmd_continue),
                        command_description(theme, model.arg_mode, "a", t.cmd_abort),
                    ],
                }],
            }],
        };
    }

    let actions_col = PopupColumn {
        title: Some(t.col_actions.into()),
        content: vec![
            command_description(theme, model.arg_mode, "m", t.cmd_merge),
            command_description(theme, model.arg_mode, "e", t.cmd_merge_edit_message),
            command_description(theme, model.arg_mode, "n", t.cmd_merge_no_commit),
            command_description(theme, model.arg_mode, "a", t.cmd_merge_absorb),
            command_description(theme, model.arg_mode, "p", t.cmd_merge_preview),
            command_description(theme, model.arg_mode, "s", t.cmd_merge_squash),
            command_description(theme, model.arg_mode, "d", t.cmd_merge_dissolve),
        ],
    };

    CommandPopupContent {
        title: t.popup_merge,
        rows: vec![PopupRow {
            columns: vec![actions_col],
        }],
    }
}
