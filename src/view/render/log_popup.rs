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

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let t = i18n::t();

    let log_col = PopupColumn {
        title: Some(t.popup_log.into()),
        content: vec![
            command_description(theme, model.arg_mode, "l", t.cmd_current),
            command_description(theme, model.arg_mode, "L", t.cmd_local_branches),
            command_description(theme, model.arg_mode, "b", t.cmd_all_branches),
            command_description(theme, model.arg_mode, "a", t.cmd_all_references),
        ],
    };

    CommandPopupContent {
        title: t.popup_log,
        rows: vec![PopupRow {
            columns: vec![log_col],
        }],
    }
}
