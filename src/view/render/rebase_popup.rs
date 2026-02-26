use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{config::Theme, model::popup::RebasePopupState};

pub fn render(state: &RebasePopupState, frame: &mut Frame, area: Rect, theme: &Theme) {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let section_style = Style::default()
        .fg(theme.section_header)
        .add_modifier(Modifier::BOLD);
    let branch_style = Style::default().fg(theme.local_branch);
    let desc_style = Style::default();

    // 2 content rows + 2 border rows
    let popup_height = 4_u16;
    let popup_width = area.width;
    let popup_area = Rect::new(
        area.x,
        area.y + area.height.saturating_sub(popup_height),
        popup_width,
        popup_height,
    );

    frame.render_widget(Clear, popup_area);

    let popup_block = Block::default()
        .title("Rebase")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.local_branch));

    frame.render_widget(popup_block, popup_area);

    let inner = Rect::new(
        popup_area.x + 1,
        popup_area.y + 1,
        popup_area.width.saturating_sub(2),
        popup_area.height.saturating_sub(2),
    );

    // Section title: "Rebase [branch] onto"
    let title_line = Line::from(vec![
        Span::styled("Rebase ", section_style),
        Span::styled(state.branch.clone(), branch_style),
        Span::styled(" onto", section_style),
    ]);
    let title_area = Rect::new(inner.x, inner.y, inner.width, 1);
    frame.render_widget(Paragraph::new(title_line), title_area);

    // Key row: " e  elsewhere"
    let key_line = Line::from(vec![
        Span::styled(" e", key_style),
        Span::styled("  elsewhere", desc_style),
    ]);
    let key_area = Rect::new(inner.x, inner.y + 1, inner.width, 1);
    frame.render_widget(Paragraph::new(key_line), key_area);
}
