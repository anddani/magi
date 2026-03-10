use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::{
    config::Theme,
    i18n,
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
    let t = i18n::t();
    let section_style = Style::default()
        .fg(theme.section_header)
        .add_modifier(Modifier::BOLD);
    let branch_style = Style::default().fg(theme.local_branch);

    if state.in_progress {
        // Rebase sequence paused on a conflict — show Continue / Skip / Abort
        return CommandPopupContent {
            title: t.section_rebasing,
            rows: vec![PopupRow {
                columns: vec![PopupColumn {
                    title: None,
                    content: vec![
                        command_description(theme, model.arg_mode, "r", t.cmd_continue),
                        command_description(theme, model.arg_mode, "s", t.cmd_skip),
                        command_description(theme, model.arg_mode, "a", t.cmd_abort),
                    ],
                }],
            }],
        };
    }

    let rebase_onto_title = Line::from(vec![
        Span::styled(t.rebase_onto_pre, section_style),
        Span::styled(state.branch.as_str(), branch_style),
        Span::styled(t.rebase_onto_post, section_style),
    ]);
    let rebase_onto_col = PopupColumn {
        title: Some(PopupColumnTitle::Styled(rebase_onto_title)),
        content: vec![command_description(
            theme,
            model.arg_mode,
            "e",
            t.cmd_elsewhere,
        )],
    };

    CommandPopupContent {
        title: t.popup_rebase,
        rows: vec![PopupRow {
            columns: vec![rebase_onto_col],
        }],
    }
}
