use super::popup_content::CommandPopupContent;

use crate::{
    config::Theme,
    i18n,
    model::Model,
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::command_description,
    },
};

pub fn content<'a>(theme: &Theme, model: &Model) -> CommandPopupContent<'a> {
    let t = i18n::t();

    let reset_col = PopupColumn {
        title: Some(t.popup_reset.into()),
        content: vec![
            command_description(theme, model.arg_mode, "b", t.cmd_branch),
            command_description(theme, model.arg_mode, "f", t.cmd_file),
        ],
    };

    let reset_this_col = PopupColumn {
        title: Some(t.col_reset_this.into()),
        content: vec![
            command_description(theme, model.arg_mode, "m", t.cmd_reset_mixed),
            command_description(theme, model.arg_mode, "s", t.cmd_reset_soft),
            command_description(theme, model.arg_mode, "h", t.cmd_reset_hard),
            command_description(theme, model.arg_mode, "k", t.cmd_reset_keep),
            command_description(theme, model.arg_mode, "i", t.cmd_reset_index),
            command_description(theme, model.arg_mode, "w", t.cmd_reset_worktree),
        ],
    };

    CommandPopupContent {
        title: t.popup_reset,
        rows: vec![PopupRow {
            columns: vec![reset_col, reset_this_col],
        }],
    }
}
