use crate::git::log::get_log_entries;
use crate::model::{Line, LineContent, Model, ViewMode};
use crate::msg::Message;

pub fn update(model: &mut Model) -> Option<Message> {
    let return_mode = model.preview_return_mode.take().unwrap_or(ViewMode::Status);
    let return_cursor = model.preview_return_cursor;

    match return_mode {
        ViewMode::Status => {
            model.view_mode = ViewMode::Status;
            model.ui_model.cursor_position = return_cursor;
            Some(Message::Refresh)
        }
        ViewMode::Log(log_type, _) => {
            // Re-enter Log view (browse mode) and restore cursor
            match get_log_entries(&model.git_info.repository, log_type) {
                Ok(entries) => {
                    let lines: Vec<Line> = entries
                        .into_iter()
                        .map(|entry| Line {
                            content: LineContent::LogLine(entry),
                            section: None,
                        })
                        .collect();
                    model.ui_model.lines = lines;
                    let max_pos = model.ui_model.lines.len().saturating_sub(1);
                    model.ui_model.cursor_position = return_cursor.min(max_pos);
                    model.ui_model.scroll_offset = 0;
                    model.view_mode = ViewMode::Log(log_type, false);
                    model.popup = None;
                }
                Err(e) => {
                    model.view_mode = ViewMode::Status;
                    model.popup = Some(crate::model::PopupContent::Error {
                        message: format!("Failed to reload log: {}", e),
                    });
                    return Some(Message::Refresh);
                }
            }
            None
        }
        ViewMode::Preview => {
            // Shouldn't happen, fall back to Status
            model.view_mode = ViewMode::Status;
            Some(Message::Refresh)
        }
    }
}
