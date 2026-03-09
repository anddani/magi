use super::popup_content::{CommandPopupContent, PopupColumn, PopupRow};

use crate::{
    config::Theme,
    model::{Model, popup::TagPopupState},
    view::render::util::command_description,
};

pub fn content<'a>(
    theme: &Theme,
    model: &Model,
    _state: &'a TagPopupState,
) -> CommandPopupContent<'a> {
    let create_col = PopupColumn {
        title: Some("Create".into()),
        content: vec![command_description(theme, model.arg_mode, "t", "tag")],
    };

    let do_col = PopupColumn {
        title: Some("Do".into()),
        content: vec![
            command_description(theme, model.arg_mode, "x", "delete"),
            command_description(theme, model.arg_mode, "p", "prune"),
        ],
    };

    CommandPopupContent {
        title: "Tag",
        rows: vec![PopupRow {
            columns: vec![create_col, do_col],
        }],
    }
}
