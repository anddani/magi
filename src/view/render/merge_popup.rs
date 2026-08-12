use super::popup_content::{CommandPopupContent, PopupColumn, PopupRow};

use crate::{
    config::Theme,
    i18n,
    model::{
        Model,
        arguments::{MergeArgument, PopupArgument},
        popup::MergePopupState,
    },
    view::render::util::{
        argument_line, argument_lines_for, argument_value_line, command_description,
    },
};

pub fn content<'a>(
    theme: &Theme,
    model: &'a Model,
    state: &'a MergePopupState,
) -> CommandPopupContent<'a> {
    let t = i18n::t();

    if state.in_progress {
        // Merge is paused on a conflict — show Continue / Abort
        return CommandPopupContent {
            title: t.popup_merging,
            rows: vec![PopupRow {
                columns: vec![PopupColumn {
                    title: None,
                    content: vec![
                        command_description(theme, model.arg_mode, "m", t.cmd_continue),
                        command_description(theme, model.arg_mode, "a", t.cmd_abort),
                    ],
                }],
            }],
        };
    }

    let merge_args = model.arguments.as_ref().and_then(|a| a.merge());

    let mut arguments = argument_lines_for::<MergeArgument>(
        theme,
        model.arg_mode,
        merge_args,
        &[MergeArgument::FfOnly, MergeArgument::NoFf],
    );

    arguments.push(argument_value_line(
        theme,
        '-',
        's',
        t.arg_merge_strategy,
        "--strategy=",
        model.arguments.as_ref().and_then(|a| a.merge_strategy()),
        model.arg_mode,
    ));

    arguments.push(argument_line(
        theme,
        MergeArgument::IgnoreSpaceChange.key(),
        t.arg_merge_ignore_space_change,
        MergeArgument::IgnoreSpaceChange.flag(),
        model.arg_mode,
        merge_args.is_some_and(|s| s.contains(&MergeArgument::IgnoreSpaceChange)),
    ));

    let arguments_col = PopupColumn {
        title: Some(t.col_arguments.into()),
        content: arguments,
    };

    let actions_col = PopupColumn {
        title: Some(t.col_actions.into()),
        content: vec![
            command_description(theme, model.arg_mode, "m", t.cmd_merge),
            command_description(theme, model.arg_mode, "e", t.cmd_merge_edit_message),
            command_description(theme, model.arg_mode, "n", t.cmd_merge_no_commit),
            command_description(theme, model.arg_mode, "a", t.cmd_merge_absorb),
            command_description(theme, model.arg_mode, "p", t.cmd_merge_preview),
            command_description(theme, model.arg_mode, "s", t.cmd_merge_squash),
            command_description(theme, model.arg_mode, "d", t.cmd_merge_dissolve),
        ],
    };

    CommandPopupContent {
        title: t.popup_merge,
        rows: vec![
            PopupRow {
                columns: vec![arguments_col],
            },
            PopupRow {
                columns: vec![actions_col],
            },
        ],
    }
}
