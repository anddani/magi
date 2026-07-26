use ratatui::text::Line;

use super::popup_content::CommandPopupContent;

use crate::{
    config::Theme,
    i18n,
    model::{Model, arguments::LogArgument},
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::{argument_lines, command_description, prefixed_argument_line},
    },
};

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let t = i18n::t();
    let args = model.arguments.as_ref().and_then(|a| a.log());
    let dim_commands = model.arg_mode || model.equals_arg_mode;

    let mut formatting: Vec<Line<'_>> = argument_lines::<LogArgument>(theme, model.arg_mode, args);
    formatting.push(prefixed_argument_line(
        theme,
        '=',
        'S',
        t.arg_log_show_signature,
        "--show-signature",
        model.equals_arg_mode,
        args.is_some_and(|a| a.contains(&LogArgument::ShowSignature)),
    ));

    let formatting_col = PopupColumn {
        title: Some(t.col_formatting.into()),
        content: formatting,
    };

    let log_col = PopupColumn {
        title: Some(t.popup_log.into()),
        content: vec![
            command_description(theme, dim_commands, "l", t.cmd_current),
            command_description(theme, dim_commands, "o", t.cmd_other),
            command_description(theme, dim_commands, "u", t.cmd_related),
            command_description(theme, dim_commands, "L", t.cmd_local_branches),
            command_description(theme, dim_commands, "b", t.cmd_all_branches),
            command_description(theme, dim_commands, "a", t.cmd_all_references),
        ],
    };

    let reflog_col = PopupColumn {
        title: Some(t.col_reflog.into()),
        content: vec![
            command_description(theme, dim_commands, "r", t.cmd_current),
            command_description(theme, dim_commands, "O", t.cmd_other),
            command_description(theme, dim_commands, "H", t.cmd_head),
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
