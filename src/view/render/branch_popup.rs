use super::popup_content::{CommandPopupContent, PopupColumn, PopupRow};
use crate::{config::Theme, view::render::util::command_description};

pub fn content(theme: &Theme) -> CommandPopupContent<'static> {
    let checkout = PopupColumn {
        title: Some("Checkout".into()),
        content: vec![
            command_description(theme, false, "b", "branch/revision"),
            command_description(theme, false, "l", "local branch"),
            command_description(theme, false, "c", "new branch"),
            command_description(theme, false, "s", "new spin-off"),
            command_description(theme, false, "w", "new worktree"),
        ],
    };

    let create = PopupColumn {
        title: Some("Create".into()),
        content: vec![
            command_description(theme, false, "n", "new branch"),
            command_description(theme, false, "S", "new spin-out"),
            command_description(theme, false, "W", "new worktree"),
            command_description(theme, false, "o", "new PR to default branch"),
            command_description(theme, false, "O", "new PR to..."),
        ],
    };

    let do_col = PopupColumn {
        title: Some("Do".into()),
        content: vec![
            command_description(theme, false, "m", "rename"),
            command_description(theme, false, "x", "delete"),
            command_description(theme, false, "X", "reset"),
        ],
    };

    CommandPopupContent {
        title: "Branch",
        rows: vec![PopupRow {
            columns: vec![checkout, create, do_col],
        }],
    }
}
