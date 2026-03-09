use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::{
    config::Theme,
    model::{Model, popup::RebasePopupState},
    view::render::{
        popup_content::{PopupColumn, PopupColumnTitle, PopupRow},
        util::command_description,
    },
};

pub fn content<'a>(
    theme: &Theme,
    model: &Model,
    state: &'a RebasePopupState,
) -> CommandPopupContent<'a> {
    let section_style = Style::default()
        .fg(theme.section_header)
        .add_modifier(Modifier::BOLD);
    let branch_style = Style::default().fg(theme.local_branch);

    if state.in_progress {
        // Rebase sequence paused on a conflict — show Continue / Skip / Abort
        return CommandPopupContent {
            title: "Rebasing",
            rows: vec![PopupRow {
                columns: vec![PopupColumn {
                    title: None,
                    content: vec![
                        command_description(theme, model.arg_mode, "r", "continue"),
                        command_description(theme, model.arg_mode, "s", "skip"),
                        command_description(theme, model.arg_mode, "a", "abort"),
                    ],
                }],
            }],
        };
    }

    let rebase_onto_title = Line::from(vec![
        Span::styled("Rebase ", section_style),
        Span::styled(state.branch.as_str(), branch_style),
        Span::styled(" onto", section_style),
    ]);
    let rebase_onto_col = PopupColumn {
        title: Some(PopupColumnTitle::Styled(rebase_onto_title)),
        content: vec![command_description(theme, model.arg_mode, "e", "elsewhere")],
    };

    CommandPopupContent {
        title: "Rebase",
        rows: vec![PopupRow {
            columns: vec![rebase_onto_col],
        }],
    }
}
