use super::popup_content::CommandPopupContent;

use crate::{
    config::Theme,
    model::Model,
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::command_description,
    },
};

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let log_col = PopupColumn {
        title: Some("Log".into()),
        content: vec![
            command_description(theme, model.arg_mode, "l", "current"),
            command_description(theme, model.arg_mode, "L", "local branches"),
            command_description(theme, model.arg_mode, "b", "all branches"),
            command_description(theme, model.arg_mode, "a", "all references"),
        ],
    };

    CommandPopupContent {
        title: "Log",
        rows: vec![PopupRow {
            columns: vec![log_col],
        }],
    }
}
