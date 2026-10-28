use ratatui::text::Line;

use super::popup_content::CommandPopupContent;

use crate::{
    config::Theme,
    i18n,
    model::{
        Model,
        arguments::{DEFAULT_LOG_LIMIT, LogArgument},
    },
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::{argument_lines, argument_value_line, command_description, prefixed_argument_line},
    },
};

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let t = i18n::t();
    let args = model.arguments.as_ref().and_then(|a| a.log());
    let dim_commands = model.arg_mode || model.equals_arg_mode;

    // Commit limiting section (top): the `-n` argument carries a value (the
    // commit count), so it is rendered with its current value like magit's `-n`
    let limit = model
        .arguments
        .as_ref()
        .and_then(|a| a.log_limit())
        .unwrap_or(DEFAULT_LOG_LIMIT)
        .to_string();
    let commit_limiting_col = PopupColumn {
        title: Some(t.col_commit_limiting.into()),
        content: vec![argument_value_line(
            theme,
            '-',
            'n',
            t.arg_log_limit,
            "-n",
            Some(&limit),
            model.arg_mode,
        )],
    };

    // LogArgument::all() order: Graph, Color, Decorate, ShowHeader, Patch, Stat
    // Insert =S after Decorate (index 2), before ShowHeader
    let mut formatting: Vec<Line<'_>> = argument_lines::<LogArgument>(theme, model.arg_mode, args);
    formatting.insert(
        3,
        prefixed_argument_line(
            theme,
            '=',
            'S',
            t.arg_log_show_signature,
            "--show-signature",
            model.equals_arg_mode,
            args.is_some_and(|a| a.contains(&LogArgument::ShowSignature)),
        ),
    );

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
                columns: vec![commit_limiting_col],
            },
            PopupRow {
                columns: vec![formatting_col],
            },
            PopupRow {
                columns: vec![log_col, reflog_col],
            },
        ],
    }
}
