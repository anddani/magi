use crate::{
    git::log::get_log_entries,
    model::{
        Line, LineContent, Model, PopupContent, ViewMode,
        arguments::{Arguments::LogArguments, LogArgument},
    },
    msg::{LogType, Message},
};

pub fn update(model: &mut Model, log_type: LogType) -> Option<Message> {
    // Graph is shown by default; only disabled when toggled off in the log popup
    let graph = match model.arguments.take() {
        Some(LogArguments(args)) => args.contains(&LogArgument::Graph),
        _ => true,
    };
    model.log_graph = graph;

    match get_log_entries(&model.git_info.repository, &log_type, graph) {
        Ok(entries) => {
            // Convert log entries to lines
            let lines: Vec<Line> = entries
                .into_iter()
                .map(|entry| Line {
                    content: LineContent::LogLine(entry),
                    section: None,
                })
                .collect();

            model.save_log_return_state();

            // Update the ui_model with log lines
            model.ui_model.lines = lines;
            model.ui_model.cursor_position = 0;
            model.ui_model.scroll_offset = 0;
            // Exit visual mode so it doesn't get carried over from Status view
            model.ui_model.visual_mode_anchor = None;

            // Switch to log view mode
            model.view_mode = ViewMode::Log(log_type, false);

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
