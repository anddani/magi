use ratatui::text::Line;

use super::popup_content::CommandPopupContent;
use crate::{
    config::Theme,
    model::{Model, arguments::FetchArgument, popup::FetchPopupState},
    view::render::{
        popup_content::{PopupColumn, PopupRow},
        util::{
            argument_lines, command_description, push_remote_description, upstream_description,
        },
    },
};

pub fn content<'a>(
    theme: &Theme,
    model: &Model,
    state: &FetchPopupState,
) -> CommandPopupContent<'a> {
    let arguments: Vec<Line<'_>> = argument_lines::<FetchArgument>(
        theme,
        model.arg_mode,
        model.arguments.as_ref().and_then(|a| a.fetch()),
    );

    let arguments_col = PopupColumn {
        title: Some("Arguments".into()),
        content: arguments,
    };

    let fetch_from_col = PopupColumn {
        title: Some("Fetch from".into()),
        content: vec![
            push_remote_description(model, theme, &state.push_remote),
            upstream_description(theme, model.arg_mode, &state.upstream),
            command_description(theme, model.arg_mode, "e", "elsewhere"),
            command_description(theme, model.arg_mode, "a", "all remotes"),
        ],
    };

    let fetch_col = PopupColumn {
        title: Some("Fetch".into()),
        content: vec![
            command_description(theme, model.arg_mode, "o", "another branch"),
            command_description(theme, model.arg_mode, "r", "explicit refspec"),
            command_description(theme, model.arg_mode, "m", "submodules"),
        ],
    };

    CommandPopupContent {
        title: "Fetch",
        rows: vec![
            PopupRow {
                columns: vec![arguments_col],
            },
            PopupRow {
                columns: vec![fetch_from_col],
            },
            PopupRow {
                columns: vec![fetch_col],
            },
        ],
    }
}
