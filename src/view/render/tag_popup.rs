use super::popup_content::{CommandPopupContent, PopupColumn, PopupRow};

use crate::{config::Theme, i18n, model::Model, view::render::util::command_description};

pub fn content<'a>(theme: &Theme, model: &Model) -> CommandPopupContent<'a> {
    let t = i18n::t();

    let create_col = PopupColumn {
        title: Some(t.col_create.into()),
        content: vec![command_description(theme, model.arg_mode, "t", t.cmd_tag)],
    };

    let do_col = PopupColumn {
        title: Some(t.col_do.into()),
        content: vec![
            command_description(theme, model.arg_mode, "x", t.cmd_delete),
            command_description(theme, model.arg_mode, "p", t.cmd_prune),
        ],
    };

    CommandPopupContent {
        title: t.popup_tag,
        rows: vec![PopupRow {
            columns: vec![create_col, do_col],
        }],
    }
}
