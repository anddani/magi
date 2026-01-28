//! Credential popup rendering for password/passphrase/etc. input.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::config::Theme;
use crate::model::popup::CredentialPopupState;

/// Render the credential input popup as a centered dialog.
pub fn render(state: &CredentialPopupState, frame: &mut Frame, area: Rect, theme: &Theme) {
    // Calculate popup dimensions
    let popup_width = 50.min(area.width.saturating_sub(4));
    let popup_height = 5; // border + title line + input line + hint + border

    // Center the popup
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    // Clear background
    frame.render_widget(Clear, popup_area);

    // Draw border with title
    let title = state.credential_type.display_title();
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.local_branch));
    frame.render_widget(block, popup_area);

    // Inner area (inside border)
    let inner = Rect::new(
        popup_area.x + 2,
        popup_area.y + 1,
        popup_area.width.saturating_sub(4),
        popup_area.height.saturating_sub(2),
    );

    // Input line with cursor
    let display_text = if state.credential_type.should_mask() {
        // Show dots for masked input
        "*".repeat(state.input_text.len())
    } else {
        state.input_text.clone()
    };

    let input_line = Line::from(vec![
        Span::styled(display_text, Style::default()),
        Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
    ]);

    let input_area = Rect::new(inner.x, inner.y, inner.width, 1);
    frame.render_widget(Paragraph::new(input_line), input_area);

    // Hint line
    let hint = "Enter to confirm, Esc to cancel";
    let hint_line = Line::from(Span::styled(hint, Style::default().fg(Color::DarkGray)));

    let hint_area = Rect::new(inner.x, inner.y + 2, inner.width, 1);
    frame.render_widget(Paragraph::new(hint_line), hint_area);
}
