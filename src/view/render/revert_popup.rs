use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::{config::Theme, model::popup::RevertPopupState};

pub fn content<'a>(theme: &Theme, state: &'a RevertPopupState) -> CommandPopupContent<'a> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();
    let grayed_style = Style::default().fg(theme.section_header);

    if state.in_progress {
        // Revert sequence is paused on a conflict — show Continue / Skip / Abort
        let key_lines = vec![
            Line::from(vec![
                Span::styled(" _", key_style),
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
        CommandPopupContent::single_column("Reverting", key_lines)
    } else {
        // Normal revert — show "Revert commit(s)", grayed if no commits selected
        let has_commits = !state.selected_commits.is_empty();
        let key_line = Line::from(vec![
            Span::styled(" _", if has_commits { key_style } else { grayed_style }),
            Span::styled(
                "  Revert commit(s)",
                if has_commits {
                    desc_style
                } else {
                    grayed_style
                },
            ),
        ]);
        CommandPopupContent::single_column("Revert commits", vec![key_line])
    }
}
