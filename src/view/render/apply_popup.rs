use super::popup_content::CommandPopupContent;

use crate::{
    config::Theme,
    i18n,
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
    let t = i18n::t();

    if state.in_progress {
        // Cherry-pick sequence is paused on a conflict — show Continue / Skip / Abort
        return CommandPopupContent {
            title: t.popup_applying,
            rows: vec![PopupRow {
                columns: vec![PopupColumn {
                    title: None,
                    content: vec![
                        command_description(theme, model.arg_mode, "A", t.cmd_continue),
                        command_description(theme, model.arg_mode, "s", t.cmd_skip),
                        command_description(theme, model.arg_mode, "a", t.cmd_abort),
                    ],
                }],
            }],
        };
    }

    CommandPopupContent {
        title: t.popup_apply,
        rows: vec![PopupRow {
            columns: vec![PopupColumn {
                title: Some(t.col_apply_here.into()),
                content: vec![
                    command_description(theme, model.arg_mode, "A", t.cmd_pick),
                    command_description(theme, model.arg_mode, "a", t.cmd_apply),
                ],
            }],
        }],
    }
}
