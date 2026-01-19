use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use super::popup_content::CommandPopupContent;
use crate::config::Theme;
use crate::model::popup::PushPopupState;

pub fn content<'a>(theme: &Theme, state: &PushPopupState) -> CommandPopupContent<'a> {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();

    let commands: Vec<Line> = if state.input_mode {
        // In input mode, show the input field
        let suggested = format!("{}/{}", state.default_remote, state.local_branch);
        let input_display = if state.input_text.is_empty() {
            // Show placeholder (remote/branch) in faded text
            vec![
                Span::styled("u", key_style),
                Span::styled(" ", desc_style),
                Span::styled(suggested, Style::default().fg(Color::DarkGray)),
                Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
            ]
        } else {
            // Show entered text with cursor
            vec![
                Span::styled("u", key_style),
                Span::styled(" ", desc_style),
                Span::styled(state.input_text.clone(), desc_style),
                Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
            ]
        };
        vec![
            Line::from(input_display),
            Line::from(vec![Span::styled(
                "  Press Enter to confirm, Esc to cancel",
                Style::default().fg(Color::DarkGray),
            )]),
        ]
    } else {
        // Normal mode - show commands
        let upstream_description = match &state.upstream {
            Some(upstream) => {
                // Upstream is set - show in remote branch color
                vec![
                    Span::styled("u", key_style),
                    Span::styled(" ", desc_style),
                    Span::styled(upstream.clone(), Style::default().fg(theme.remote_branch)),
                ]
            }
            None => {
                // No upstream - show suggestion with ", creating it"
                vec![
                    Span::styled("u", key_style),
                    Span::styled(" ${upstream}, creating it", desc_style),
                ]
            }
        };
        vec![Line::from(upstream_description)]
    };

    let arguments: Vec<Line> = vec![];

    CommandPopupContent::two_columns("Push", "Commands", commands, "Arguments", arguments)
}
