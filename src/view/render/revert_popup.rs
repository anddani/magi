use ratatui::text::Line;

use super::popup_content::CommandPopupContent;

use crate::{
    config::Theme,
    i18n,
    model::{Model, arguments::RevertArgument, popup::RevertPopupState},
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::{argument_lines, argument_value_line, command_description},
    },
};

pub fn content<'a>(
    theme: &Theme,
    model: &Model,
    state: &'a RevertPopupState,
) -> CommandPopupContent<'a> {
    let t = i18n::t();

    if state.in_progress {
        // Revert sequence is paused on a conflict — show Continue / Skip / Abort
        return CommandPopupContent {
            title: t.section_reverting,
            rows: vec![PopupRow {
                columns: vec![PopupColumn {
                    title: None,
                    content: vec![
                        command_description(theme, model.arg_mode, "_", t.cmd_continue),
                        command_description(theme, model.arg_mode, "s", t.cmd_skip),
                        command_description(theme, model.arg_mode, "a", t.cmd_abort),
                    ],
                }],
            }],
        };
    }

    let mut arguments: Vec<Line<'_>> = vec![argument_value_line(
        theme,
        'm',
        t.arg_revert_mainline,
        "--mainline=",
        state.mainline.as_deref(),
        model.arg_mode,
    )];

    arguments.extend(argument_lines::<RevertArgument>(
        theme,
        model.arg_mode,
        model.arguments.as_ref().and_then(|a| a.revert()),
    ));

    let arguments_col = PopupColumn {
        title: Some(t.col_arguments.into()),
        content: arguments,
    };

    CommandPopupContent {
        title: t.popup_revert,
        rows: vec![
            PopupRow {
                columns: vec![arguments_col],
            },
            PopupRow {
                columns: vec![PopupColumn {
                    title: Some(t.col_actions.into()),
                    content: vec![
                        command_description(theme, model.arg_mode, "_", t.cmd_revert_commits),
                        command_description(theme, model.arg_mode, "s", t.cmd_skip),
                    ],
                }],
            },
        ],
    }
}
