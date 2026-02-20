use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line as TextLine, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    model::{
        Model, Toast, ToastStyle,
        popup::{PopupContent, PopupContentCommand},
    },
    view::render::popup_content::CommandPopupContent,
};

mod branch_popup;
mod commit_popup;
mod commit_select_popup;
mod credential_popup;
mod fetch_popup;
mod help_popup;
mod input_popup;
mod log_popup;
mod popup_content;
mod pull_popup;
mod push_popup;
mod select_popup;
mod util;

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
    model: &Model,
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
            let content = match command {
                PopupContentCommand::Commit => commit_popup::content(theme, model),
                PopupContentCommand::Push(state) => push_popup::content(theme, model, state),
                PopupContentCommand::Fetch(state) => fetch_popup::content(theme, model, state),
                PopupContentCommand::Pull(state) => pull_popup::content(theme, model, state),
                PopupContentCommand::Branch => branch_popup::content(theme),
                PopupContentCommand::Log => log_popup::content(theme),

                // Select popup uses custom rendering, not the column layout
                PopupContentCommand::Select(state) => {
                    select_popup::render(state, frame, area, theme);
                    return;
                }

                // Commit select popup uses custom rendering with log line formatting
                PopupContentCommand::CommitSelect(state) => {
                    commit_select_popup::render(state, frame, area, theme);
                    return;
                }
            };
            render_command_popup(frame, area, theme, &content);
        }
        PopupContent::Credential(state) => {
            credential_popup::render(state, frame, area, theme);
        }
        PopupContent::Confirm(state) => {
            render_confirm_popup(&state.message, frame, area, theme);
        }
        PopupContent::Input(state) => {
            input_popup::render(state, frame, area, theme);
        }
        PopupContent::Help => render_command_popup(frame, area, theme, &help_popup::content(theme)),
    }
}

/// Render an error popup (centered)
fn render_error_popup(message: &str, frame: &mut Frame, area: Rect, theme: &crate::config::Theme) {
    let title = "Error";
    let border_color = theme.diff_deletion;

    // Split message into lines
    let message_lines: Vec<&str> = message.lines().collect();
    let line_count = message_lines.len();

    // Calculate popup size based on content
    let max_line_width = message_lines
        .iter()
        .map(|line| line.len())
        .max()
        .unwrap_or(0);
    let content_width = max_line_width.max(title.len()) + 4; // padding
    let popup_width = (content_width as u16).clamp(30, area.width.saturating_sub(4));
    let popup_height = (line_count + 4) as u16; // border + content lines + empty line + hint + border

    let popup_area = centered_rect(popup_width, popup_height, area);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    // Build popup content with hint
    let hint = "Press Enter or Esc to dismiss";
    let mut popup_text: Vec<TextLine> = message_lines.into_iter().map(TextLine::from).collect();
    popup_text.push(TextLine::from(""));
    popup_text.push(TextLine::from(Span::styled(
        hint,
        Style::default().fg(Color::DarkGray),
    )));

    let popup_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let popup_paragraph = Paragraph::new(popup_text).block(popup_block);

    frame.render_widget(popup_paragraph, popup_area);
}

/// Render a confirmation popup (centered, y/n)
fn render_confirm_popup(
    message: &str,
    frame: &mut Frame,
    area: Rect,
    theme: &crate::config::Theme,
) {
    let title = "Confirm";
    let border_color = theme.section_header;

    let content_width = message.len().max(title.len()) + 4;
    let popup_width = (content_width as u16).clamp(30, area.width.saturating_sub(4));
    let popup_height = 3; // border + message + border

    let popup_area = centered_rect(popup_width, popup_height, area);

    frame.render_widget(Clear, popup_area);

    let popup_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let popup_paragraph = Paragraph::new(TextLine::from(message)).block(popup_block);

    frame.render_widget(popup_paragraph, popup_area);
}

/// Render a command popup (bottom half of screen)
fn render_command_popup(
    frame: &mut Frame,
    area: Rect,
    theme: &crate::config::Theme,
    content: &CommandPopupContent,
) {
    let column_title_style = Style::default()
        .fg(theme.section_header)
        .add_modifier(Modifier::BOLD);

    // Calculate popup dimensions
    let popup_height = (content.max_content_height() + 2) as u16; // +2 for border
    let popup_width = area.width;
    let popup_area = bottom_half_rect(popup_width, popup_height, area);

    // Clear the area behind the popup
    frame.render_widget(Clear, popup_area);

    // Render outer block with title
    let popup_block = Block::default()
        .title(content.title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.local_branch));

    frame.render_widget(popup_block, popup_area);

    // Inner area for content (inside the border)
    let inner_area = Rect::new(
        popup_area.x + 1,
        popup_area.y + 1,
        popup_area.width.saturating_sub(2),
        popup_area.height.saturating_sub(2),
    );

    // Create layout constraints based on number of columns
    let constraints: Vec<Constraint> = content
        .columns
        .iter()
        .map(|_| Constraint::Ratio(1, content.columns.len() as u32))
        .collect();

    let column_areas = Layout::horizontal(constraints).split(inner_area);

    // Render each column
    for (i, column) in content.columns.iter().enumerate() {
        let mut column_content: Vec<TextLine> = Vec::new();

        // Add column title if present
        if let Some(title) = column.title {
            column_content.push(TextLine::from(Span::styled(title, column_title_style)));
        }

        column_content.extend(column.content.clone());

        let paragraph = Paragraph::new(column_content);
        frame.render_widget(paragraph, column_areas[i]);
    }
}
