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
        util::{
            argument_line, argument_lines, argument_value_line, command_description,
            prefixed_argument_line,
        },
    },
};

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let t = i18n::t();
    let args = model.arguments.as_ref().and_then(|a| a.log());
    let dim_commands = model.arg_mode || model.equals_arg_mode;

    // Commit limiting section (top): the `-n`, `-A`, `-F`, `-G`, `-S` and `-L`
    // arguments carry values, so they are rendered with their current value
    // like magit's `-n`/`-A`/`-F`/`-G`/`-S`/`-L`
    let limit = model
        .arguments
        .as_ref()
        .and_then(|a| a.log_limit())
        .unwrap_or(DEFAULT_LOG_LIMIT)
        .to_string();
    let author = model.arguments.as_ref().and_then(|a| a.log_author());
    let grep = model.arguments.as_ref().and_then(|a| a.log_grep());
    let search_changes = model
        .arguments
        .as_ref()
        .and_then(|a| a.log_search_changes());
    let search_occurrences = model
        .arguments
        .as_ref()
        .and_then(|a| a.log_search_occurrences());
    let trace_lines = model.arguments.as_ref().and_then(|a| a.log_trace_lines());
    let commit_limiting_col = PopupColumn {
        title: Some(t.col_commit_limiting.into()),
        content: vec![
            argument_value_line(
                theme,
                '-',
                'n',
                t.arg_log_limit,
                "-n",
                Some(&limit),
                model.arg_mode,
            ),
            argument_value_line(
                theme,
                '-',
                'A',
                t.arg_log_author,
                "--author=",
                author,
                model.arg_mode,
            ),
            argument_value_line(
                theme,
                '-',
                'F',
                t.arg_log_grep,
                "--grep=",
                grep,
                model.arg_mode,
            ),
            argument_value_line(
                theme,
                '-',
                'G',
                t.arg_log_search_changes,
                "-G",
                search_changes,
                model.arg_mode,
            ),
            argument_value_line(
                theme,
                '-',
                'S',
                t.arg_log_search_occurrences,
                "-S",
                search_occurrences,
                model.arg_mode,
            ),
            argument_value_line(
                theme,
                '-',
                'L',
                t.arg_log_trace_lines,
                "-L",
                trace_lines,
                model.arg_mode,
            ),
        ],
    };

    // History simplification section: the `-D` toggle lives here (not in
    // Formatting), rendered with the `-` prefix like the other toggles; `--`
    // carries a value (the pathspec), rendered like the other value args
    let files = model.arguments.as_ref().and_then(|a| a.log_files());
    let history_simplification_col = PopupColumn {
        title: Some(t.col_history_simplification.into()),
        content: vec![
            argument_line(
                theme,
                'D',
                t.arg_log_simplify_by_decoration,
                "--simplify-by-decoration",
                model.arg_mode,
                args.is_some_and(|a| a.contains(&LogArgument::SimplifyByDecoration)),
            ),
            argument_value_line(
                theme,
                '-',
                '-',
                t.arg_log_files,
                "--",
                files,
                model.arg_mode,
            ),
            argument_line(
                theme,
                'f',
                t.arg_log_follow,
                "--follow",
                model.arg_mode,
                args.is_some_and(|a| a.contains(&LogArgument::Follow)),
            ),
        ],
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
                columns: vec![history_simplification_col],
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
