use ratatui::text::Line;

use super::popup_content::{CommandPopupContent, PopupColumn, PopupRow};

use crate::{
    config::Theme,
    model::{Model, arguments::StashArgument},
    view::render::util::{argument_lines, command_description},
};

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let arguments: Vec<Line<'_>> = argument_lines::<StashArgument>(
        theme,
        model.arg_mode,
        model.arguments.as_ref().and_then(|a| a.stash()),
    );

    let arguments_col = PopupColumn {
        title: Some("Arguments".into()),
        content: arguments,
    };

    let stash = PopupColumn {
        title: Some("Stash".into()),
        content: vec![
            command_description(theme, model.arg_mode, "z", "both"),
            command_description(theme, model.arg_mode, "i", "index"),
            command_description(theme, model.arg_mode, "w", "worktree"),
        ],
    };

    let use_col = PopupColumn {
        title: Some("Use".into()),
        content: vec![
            command_description(theme, model.arg_mode, "a", "apply"),
            command_description(theme, model.arg_mode, "p", "pop"),
            command_description(theme, model.arg_mode, "k", "drop"),
        ],
    };

    CommandPopupContent {
        title: "Stash",
        rows: vec![
            PopupRow {
                columns: vec![arguments_col],
            },
            PopupRow {
                columns: vec![stash, use_col],
            },
        ],
    }
}
