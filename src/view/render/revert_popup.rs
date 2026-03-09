use super::popup_content::CommandPopupContent;

use crate::{
    config::Theme,
    model::{Model, popup::RevertPopupState},
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::command_description,
    },
};

pub fn content<'a>(
    theme: &Theme,
    model: &Model,
    state: &'a RevertPopupState,
) -> CommandPopupContent<'a> {
    if state.in_progress {
        // Revert sequence is paused on a conflict — show Continue / Skip / Abort
        return CommandPopupContent {
            title: "Reverting",
            rows: vec![PopupRow {
                columns: vec![PopupColumn {
                    title: None,
                    content: vec![
                        command_description(theme, model.arg_mode, "_", "continue"),
                        command_description(theme, model.arg_mode, "s", "skip"),
                        command_description(theme, model.arg_mode, "a", "abort"),
                    ],
                }],
            }],
        };
    }

    CommandPopupContent {
        title: "Revert",
        rows: vec![PopupRow {
            columns: vec![PopupColumn {
                title: Some("Actions".into()),
                content: vec![
                    command_description(theme, model.arg_mode, "_", "Revert commit(s)"),
                    command_description(theme, model.arg_mode, "s", "skip"),
                ],
            }],
        }],
    }
}
