use ratatui::text::Line;

use super::popup_content::CommandPopupContent;

use crate::{
    config::Theme,
    i18n,
    model::{Model, arguments::LogArgument},
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::{argument_lines, command_description},
    },
};

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let t = i18n::t();

    let formatting: Vec<Line<'_>> = argument_lines::<LogArgument>(
        theme,
        model.arg_mode,
        model.arguments.as_ref().and_then(|a| a.log()),
    );

    let formatting_col = PopupColumn {
        title: Some(t.col_formatting.into()),
        content: formatting,
    };

    let log_col = PopupColumn {
        title: Some(t.popup_log.into()),
        content: vec![
            command_description(theme, model.arg_mode, "l", t.cmd_current),
            command_description(theme, model.arg_mode, "o", t.cmd_other),
            command_description(theme, model.arg_mode, "u", t.cmd_related),
            command_description(theme, model.arg_mode, "L", t.cmd_local_branches),
            command_description(theme, model.arg_mode, "b", t.cmd_all_branches),
            command_description(theme, model.arg_mode, "a", t.cmd_all_references),
        ],
    };

    let reflog_col = PopupColumn {
        title: Some(t.col_reflog.into()),
        content: vec![
            command_description(theme, model.arg_mode, "r", t.cmd_current),
            command_description(theme, model.arg_mode, "O", t.cmd_other),
        ],
    };

    CommandPopupContent {
        title: t.popup_log,
        rows: vec![
            PopupRow {
                columns: vec![formatting_col],
            },
            PopupRow {
                columns: vec![log_col, reflog_col],
            },
        ],
    }
}
