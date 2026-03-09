use super::popup_content::{CommandPopupContent, PopupColumn, PopupRow};

use crate::{
    config::Theme,
    model::{Model, popup::MergePopupState},
    view::render::util::command_description,
};

pub fn content<'a>(
    theme: &Theme,
    model: &Model,
    state: &'a MergePopupState,
) -> CommandPopupContent<'a> {
    if state.in_progress {
        // Merge is paused on a conflict — show Continue / Abort
        return CommandPopupContent {
            title: "Merging",
            rows: vec![PopupRow {
                columns: vec![PopupColumn {
                    title: None,
                    content: vec![
                        command_description(theme, model.arg_mode, "m", "continue"),
                        command_description(theme, model.arg_mode, "m", "abort"),
                    ],
                }],
            }],
        };
    }

    let actions_col = PopupColumn {
        title: Some("Actions".into()),
        content: vec![command_description(theme, model.arg_mode, "m", "merge")],
    };

    CommandPopupContent {
        title: "Merge",
        rows: vec![PopupRow {
            columns: vec![actions_col],
        }],
    }
}
