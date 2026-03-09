use super::popup_content::CommandPopupContent;

use crate::{
    config::Theme,
    model::{Model, popup::ApplyPopupState},
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::command_description,
    },
};

pub fn content<'a>(
    theme: &Theme,
    model: &Model,
    state: &'a ApplyPopupState,
) -> CommandPopupContent<'a> {
    if state.in_progress {
        // Cherry-pick sequence is paused on a conflict — show Continue / Skip / Abort
        return CommandPopupContent {
            title: "Applying",
            rows: vec![PopupRow {
                columns: vec![PopupColumn {
                    title: None,
                    content: vec![
                        command_description(theme, model.arg_mode, "A", "continue"),
                        command_description(theme, model.arg_mode, "s", "skip"),
                        command_description(theme, model.arg_mode, "a", "abort"),
                    ],
                }],
            }],
        };
    }

    CommandPopupContent {
        title: "Apply",
        rows: vec![PopupRow {
            columns: vec![PopupColumn {
                title: Some("Apply here".into()),
                content: vec![command_description(theme, model.arg_mode, "A", "pick")],
            }],
        }],
    }
}
