use super::popup_content::CommandPopupContent;

use crate::{
    config::Theme,
    model::Model,
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::command_description,
    },
};

pub fn content<'a>(theme: &Theme, model: &Model) -> CommandPopupContent<'a> {
    let reset_col = PopupColumn {
        title: Some("Reset".into()),
        content: vec![
            command_description(theme, model.arg_mode, "b", "branch"),
            command_description(theme, model.arg_mode, "f", "file"),
        ],
    };

    let reset_this_col = PopupColumn {
        title: Some("Reset this".into()),
        content: vec![
            command_description(theme, model.arg_mode, "m", "mixed    (HEAD and index)"),
            command_description(theme, model.arg_mode, "s", "soft     (HEAD only)"),
            command_description(
                theme,
                model.arg_mode,
                "h",
                "hard     (HEAD, index and worktree)",
            ),
            command_description(
                theme,
                model.arg_mode,
                "k",
                "keep     (HEAD and index, keeping uncommitted)",
            ),
            command_description(theme, model.arg_mode, "i", "index    (only)"),
            command_description(theme, model.arg_mode, "w", "worktree (only)"),
        ],
    };

    CommandPopupContent {
        title: "Reset",
        rows: vec![PopupRow {
            columns: vec![reset_col, reset_this_col],
        }],
    }
}
