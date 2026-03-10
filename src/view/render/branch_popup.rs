use super::popup_content::{CommandPopupContent, PopupColumn, PopupRow};
use crate::{config::Theme, i18n, view::render::util::command_description};

pub fn content(theme: &Theme) -> CommandPopupContent<'static> {
    let t = i18n::t();

    let checkout = PopupColumn {
        title: Some(t.col_checkout.into()),
        content: vec![
            command_description(theme, false, "b", t.cmd_branch_revision),
            command_description(theme, false, "l", t.cmd_local_branch),
            command_description(theme, false, "c", t.cmd_new_branch),
            command_description(theme, false, "s", t.cmd_new_spinoff),
            command_description(theme, false, "w", t.cmd_new_worktree),
        ],
    };

    let create = PopupColumn {
        title: Some(t.col_create.into()),
        content: vec![
            command_description(theme, false, "n", t.cmd_new_branch),
            command_description(theme, false, "S", t.cmd_new_spinout),
            command_description(theme, false, "W", t.cmd_new_worktree),
            command_description(theme, false, "o", t.cmd_new_pr_default),
            command_description(theme, false, "O", t.cmd_new_pr_to),
        ],
    };

    let do_col = PopupColumn {
        title: Some(t.col_do.into()),
        content: vec![
            command_description(theme, false, "m", t.cmd_rename),
            command_description(theme, false, "x", t.cmd_delete),
            command_description(theme, false, "X", t.cmd_reset),
        ],
    };

    CommandPopupContent {
        title: t.popup_branch,
        rows: vec![PopupRow {
            columns: vec![checkout, create, do_col],
        }],
    }
}
