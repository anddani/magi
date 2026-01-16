use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line as TextLine, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::model::{
    popup::{PopupContent, PopupContentCommand},
    Toast, ToastStyle,
};

mod help_popup;

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

/// Calculate a rectangle in the bottom half of the screen, centered horizontally
fn bottom_half_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + area.height.saturating_sub(height);
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

/// Render a modal popup overlay (centered, requires user action)
pub fn render_popup(
    popup: &PopupContent,
    frame: &mut Frame,
    area: Rect,
    theme: &crate::config::Theme,
) {
    match popup {
        PopupContent::Error { message } => {
            render_error_popup(message, frame, area, theme);
        }
        PopupContent::Command(command) => {
            render_command_popup(frame, area, theme, command);
        }
    }
}

/// Render an error popup (centered)
fn render_error_popup(message: &str, frame: &mut Frame, area: Rect, theme: &crate::config::Theme) {
    let title = "Error";
    let border_color = theme.diff_deletion;

    // Calculate popup size based on content
    let content_width = message.len().max(title.len()) + 4; // padding
    let popup_width = (content_width as u16).clamp(30, area.width.saturating_sub(4));
    let popup_height = 5; // title bar + content + border + hint

    let popup_area = centered_rect(popup_width, popup_height, area);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    // Build popup content with hint
    let hint = "Press Enter or Esc to dismiss";
    let popup_text = vec![
        TextLine::from(message),
        TextLine::from(""),
        TextLine::from(Span::styled(hint, Style::default().fg(Color::DarkGray))),
    ];

    let popup_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let popup_paragraph = Paragraph::new(popup_text).block(popup_block);

    frame.render_widget(popup_paragraph, popup_area);
}

/// Render the help popup showing keybindings (bottom half of screen)
fn render_command_popup(
    frame: &mut Frame,
    area: Rect,
    theme: &crate::config::Theme,
    command: &PopupContentCommand,
) {
    // Define keybindings grouped by category
    let (title, content) = match command {
        PopupContentCommand::Help => help_popup::content(theme),
    };

    let popup_height = (content.len() + 2) as u16; // +2 for border
    let popup_width = area.width;

    let popup_area = bottom_half_rect(popup_width, popup_height, area);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    let popup_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.local_branch));

    let popup_paragraph = Paragraph::new(content).block(popup_block);

    frame.render_widget(popup_paragraph, popup_area);
}
