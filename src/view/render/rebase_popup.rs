use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::{config::Theme, model::popup::RebasePopupState};

pub fn content<'a>(theme: &Theme, state: &'a RebasePopupState) -> CommandPopupContent<'a> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let section_style = Style::default()
        .fg(theme.section_header)
        .add_modifier(Modifier::BOLD);
    let branch_style = Style::default().fg(theme.local_branch);
    let desc_style = Style::default();

    if state.in_progress {
        // Rebase sequence paused on a conflict — show Continue / Skip / Abort
        let key_lines = vec![
            Line::from(vec![
                Span::styled(" r", key_style),
                Span::styled("  Continue", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" s", key_style),
                Span::styled("  Skip", desc_style),
            ]),
            Line::from(vec![
                Span::styled(" a", key_style),
                Span::styled("  Abort", desc_style),
            ]),
        ];
        CommandPopupContent::single_column("Rebasing", key_lines)
    } else {
        let title_line = Line::from(vec![
            Span::styled("Rebase ", section_style),
            Span::styled(state.branch.as_str(), branch_style),
            Span::styled(" onto", section_style),
        ]);

        let key_line = Line::from(vec![
            Span::styled(" e", key_style),
            Span::styled("  elsewhere", desc_style),
        ]);

        CommandPopupContent::single_column("Rebase", vec![title_line, key_line])
    }
}
