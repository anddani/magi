use crate::{
    git::log::get_log_entries,
    model::{Line, LineContent, Model, PopupContent, ViewMode},
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    match get_log_entries(&model.git_info.repository) {
        Ok(entries) => {
            // Convert log entries to lines
            let lines: Vec<Line> = entries
                .into_iter()
                .map(|entry| Line {
                    content: LineContent::LogLine(entry),
                    section: None,
                })
                .collect();

            // Update the ui_model with log lines
            model.ui_model.lines = lines;
            model.ui_model.cursor_position = 0;
            model.ui_model.scroll_offset = 0;

            // Switch to log view mode
            model.view_mode = ViewMode::Log;

            // Dismiss the log popup
            model.popup = None;

            None
        }
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to get log: {}", e),
            });
            None
        }
    }
}
