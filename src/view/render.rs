use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line as TextLine, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::model::{DialogContent, Toast, ToastStyle};

/// Calculate a centered rectangle within the given area
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

/// Calculate a rectangle in the bottom-right corner
fn bottom_right_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width + 1);
    let y = area.y + area.height.saturating_sub(height + 1);
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

/// Render a toast notification in the bottom-right corner
pub fn render_toast(toast: &Toast, frame: &mut Frame, area: Rect, theme: &crate::config::Theme) {
    let border_color = match toast.style {
        ToastStyle::Success => theme.staged_status,
        ToastStyle::Info => theme.local_branch,
        ToastStyle::Warning => theme.section_header,
    };

    // Calculate toast size based on content
    let content_width = toast.message.len() + 4; // padding
    let toast_width = (content_width as u16).clamp(20, area.width.saturating_sub(4));
    let toast_height = 3; // border + content + border

    let toast_area = bottom_right_rect(toast_width, toast_height, area);

    // Clear the area behind the toast
    frame.render_widget(Clear, toast_area);

    let toast_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let toast_paragraph = Paragraph::new(toast.message.as_str()).block(toast_block);

    frame.render_widget(toast_paragraph, toast_area);
}

/// Render a modal dialog overlay (centered, requires user action)
pub fn render_dialog(
    dialog: &DialogContent,
    frame: &mut Frame,
    area: Rect,
    theme: &crate::config::Theme,
) {
    let (title, content, border_color) = match dialog {
        DialogContent::Error { message } => ("Error", message.as_str(), theme.diff_deletion),
    };

    // Calculate dialog size based on content
    let content_width = content.len().max(title.len()) + 4; // padding
    let dialog_width = (content_width as u16).clamp(30, area.width.saturating_sub(4));
    let dialog_height = 5; // title bar + content + border + hint

    let dialog_area = centered_rect(dialog_width, dialog_height, area);

    // Clear the area behind the dialog
    frame.render_widget(Clear, dialog_area);

    // Build dialog content with hint
    let hint = "Press Enter or Esc to dismiss";
    let dialog_text = vec![
        TextLine::from(content),
        TextLine::from(""),
        TextLine::from(Span::styled(hint, Style::default().fg(Color::DarkGray))),
    ];

    let dialog_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let dialog_paragraph = Paragraph::new(dialog_text).block(dialog_block);

    frame.render_widget(dialog_paragraph, dialog_area);
}
