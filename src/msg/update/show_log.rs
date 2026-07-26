use crate::{
    git::log::get_log_entries,
    model::{
        Line, LineContent, Model, PopupContent, ViewMode,
        arguments::{Arguments::LogArguments, LogArgument},
    },
    msg::{LogType, Message},
};

pub fn update(model: &mut Model, log_type: LogType) -> Option<Message> {
    // Graph and refnames are shown by default; only disabled when toggled off
    // in the log popup
    let (graph, color, decorate, show_header, show_signature) = match model.arguments.take() {
        Some(LogArguments(args)) => (
            args.contains(&LogArgument::Graph),
            args.contains(&LogArgument::Color),
            args.contains(&LogArgument::Decorate),
            args.contains(&LogArgument::ShowHeader),
            args.contains(&LogArgument::ShowSignature),
        ),
        _ => (true, false, true, false, false),
    };
    let reflog = matches!(
        log_type,
        LogType::Reflog | LogType::ReflogOther(_) | LogType::Stashes
    );
    // Reflogs cannot be drawn as a graph (git rejects --graph with --walk-reflogs)
    let graph = graph && !reflog;
    // Like Magit's reflog format, headers are never shown for reflogs
    let show_header = show_header && !reflog;
    match get_log_entries(
        &model.git_info.repository,
        &log_type,
        graph,
        color,
        decorate,
        show_header,
        show_signature,
    ) {
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
            model.view_mode = ViewMode::Log {
                log_type,
                picking: false,
                graph,
                color,
                decorate,
                show_header,
                show_signature,
            };

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
