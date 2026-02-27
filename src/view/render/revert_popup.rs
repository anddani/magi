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
        // Normal revert — both actions grayed out when no commits are selected
        let has_commits = !state.selected_commits.is_empty();
        let (act_key, act_desc) = if has_commits {
            (key_style, desc_style)
        } else {
            (grayed_style, grayed_style)
        };
        let key_lines = vec![
            Line::from(vec![
                Span::styled(" _", act_key),
                Span::styled("  Revert commit(s)", act_desc),
            ]),
            Line::from(vec![
                Span::styled(" v", act_key),
                Span::styled("  Revert no commit", act_desc),
            ]),
        ];
        CommandPopupContent::single_column("Revert commits", key_lines)
    }
}
