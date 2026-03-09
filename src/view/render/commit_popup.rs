use ratatui::text::Line;

use super::popup_content::CommandPopupContent;
use crate::{
    config::Theme,
    model::{Model, arguments::CommitArgument},
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::{argument_lines, command_description},
    },
};

pub fn content(theme: &Theme, model: &Model) -> CommandPopupContent<'static> {
    let arguments: Vec<Line<'_>> = argument_lines::<CommitArgument>(
        theme,
        model.arg_mode,
        model.arguments.as_ref().and_then(|a| a.commit()),
    );

    let arguments_col = PopupColumn {
        title: Some("Arguments".into()),
        content: arguments,
    };

    let create_col = PopupColumn {
        title: Some("Create".into()),
        content: vec![command_description(theme, model.arg_mode, "c", "commit")],
    };

    let edit_head_col = PopupColumn {
        title: Some("Edit HEAD".into()),
        content: vec![
            command_description(theme, model.arg_mode, "e", "extend"),
            Line::from(""),
            command_description(theme, model.arg_mode, "a", "amend"),
            Line::from(""),
            command_description(theme, model.arg_mode, "w", "reword"),
        ],
    };

    let edit_col = PopupColumn {
        title: Some("Edit".into()),
        content: vec![
            command_description(theme, model.arg_mode, "f", "fixup"),
            command_description(theme, model.arg_mode, "s", "squash"),
            command_description(theme, model.arg_mode, "A", "alter"),
            command_description(theme, model.arg_mode, "n", "augment"),
            command_description(theme, model.arg_mode, "W", "revise"),
        ],
    };

    CommandPopupContent {
        title: "Commit",
        rows: vec![
            PopupRow {
                columns: vec![arguments_col],
            },
            PopupRow {
                columns: vec![create_col, edit_head_col, edit_col],
            },
        ],
    }
}
