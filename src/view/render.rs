use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
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

/// Render a modal dialog overlay (centered, requires user action)
pub fn render_dialog(
    dialog: &DialogContent,
    frame: &mut Frame,
    area: Rect,
    theme: &crate::config::Theme,
) {
    match dialog {
        DialogContent::Error { message } => {
            render_error_dialog(message, frame, area, theme);
        }
        DialogContent::Help => {
            render_help_popup(frame, area, theme);
        }
    }
}

/// Render an error dialog (centered)
fn render_error_dialog(
    message: &str,
    frame: &mut Frame,
    area: Rect,
    theme: &crate::config::Theme,
) {
    let title = "Error";
    let border_color = theme.diff_deletion;

    // Calculate dialog size based on content
    let content_width = message.len().max(title.len()) + 4; // padding
    let dialog_width = (content_width as u16).clamp(30, area.width.saturating_sub(4));
    let dialog_height = 5; // title bar + content + border + hint

    let dialog_area = centered_rect(dialog_width, dialog_height, area);

    // Clear the area behind the dialog
    frame.render_widget(Clear, dialog_area);

    // Build dialog content with hint
    let hint = "Press Enter or Esc to dismiss";
    let dialog_text = vec![
        TextLine::from(message),
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

/// Render the help popup showing keybindings (bottom half of screen)
fn render_help_popup(frame: &mut Frame, area: Rect, theme: &crate::config::Theme) {
    let key_style = Style::default()
        .fg(theme.local_branch)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default();
    let section_style = Style::default()
        .fg(theme.section_header)
        .add_modifier(Modifier::BOLD);
    let hint_style = Style::default().fg(Color::DarkGray);

    // Define keybindings grouped by category
    let keybindings: Vec<TextLine> = vec![
        // Navigation section
        TextLine::from(Span::styled("Navigation", section_style)),
        TextLine::from(vec![
            Span::styled("  j/Down  ", key_style),
            Span::styled("Move down", desc_style),
        ]),
        TextLine::from(vec![
            Span::styled("  k/Up    ", key_style),
            Span::styled("Move up", desc_style),
        ]),
        TextLine::from(vec![
            Span::styled("  Ctrl-d  ", key_style),
            Span::styled("Half page down", desc_style),
        ]),
        TextLine::from(vec![
            Span::styled("  Ctrl-u  ", key_style),
            Span::styled("Half page up", desc_style),
        ]),
        TextLine::from(vec![
            Span::styled("  Ctrl-e  ", key_style),
            Span::styled("Scroll line down", desc_style),
        ]),
        TextLine::from(vec![
            Span::styled("  Ctrl-y  ", key_style),
            Span::styled("Scroll line up", desc_style),
        ]),
        TextLine::from(vec![
            Span::styled("  Tab     ", key_style),
            Span::styled("Toggle section collapse/expand", desc_style),
        ]),
        TextLine::from(""),
        // Actions section
        TextLine::from(Span::styled("Actions", section_style)),
        TextLine::from(vec![
            Span::styled("  c       ", key_style),
            Span::styled("Commit", desc_style),
        ]),
        TextLine::from(vec![
            Span::styled("  S       ", key_style),
            Span::styled("Stage all modified files", desc_style),
        ]),
        TextLine::from(vec![
            Span::styled("  U       ", key_style),
            Span::styled("Unstage all files", desc_style),
        ]),
        TextLine::from(vec![
            Span::styled("  V       ", key_style),
            Span::styled("Enter visual selection mode", desc_style),
        ]),
        TextLine::from(""),
        // General section
        TextLine::from(Span::styled("General", section_style)),
        TextLine::from(vec![
            Span::styled("  q       ", key_style),
            Span::styled("Quit", desc_style),
        ]),
        TextLine::from(vec![
            Span::styled("  Ctrl-r  ", key_style),
            Span::styled("Refresh", desc_style),
        ]),
        TextLine::from(vec![
            Span::styled("  ?       ", key_style),
            Span::styled("Show this help", desc_style),
        ]),
        TextLine::from(""),
        TextLine::from(Span::styled("Press Enter or Esc to dismiss", hint_style)),
    ];

    let popup_height = (keybindings.len() + 2) as u16; // +2 for border
    let popup_width = area.width;

    let popup_area = bottom_half_rect(popup_width, popup_height, area);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    let popup_block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.local_branch));

    let popup_paragraph = Paragraph::new(keybindings).block(popup_block);

    frame.render_widget(popup_paragraph, popup_area);
}
