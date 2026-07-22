use super::popup_content::{CommandPopupContent, PopupColumn, PopupRow};
use crate::{config::Theme, i18n, view::render::util::command_description};

pub fn content(theme: &Theme) -> CommandPopupContent<'static> {
    let t = i18n::t();

    let create_new = PopupColumn {
        title: Some(t.col_create_new.into()),
        content: vec![command_description(theme, false, "b", t.cmd_worktree)],
    };

    CommandPopupContent {
        title: t.popup_worktree,
        rows: vec![PopupRow {
            columns: vec![create_new],
        }],
    }
}
