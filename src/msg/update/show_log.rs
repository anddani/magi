use crate::{
    git::log::get_log_entries,
    i18n,
    model::{
        Line, LineContent, LogOptions, Model, PopupContent, ViewMode,
        arguments::{Arguments::LogArguments, LogArgument},
    },
    msg::{LogType, Message},
};

pub fn update(model: &mut Model, log_type: LogType) -> Option<Message> {
    // Graph and refnames are shown by default (via LogOptions::default); only
    // disabled when toggled off in the log popup
    let mut options = match model.arguments.take() {
        Some(LogArguments {
            args,
            limit,
            author,
            grep,
            search_changes,
            search_occurrences,
            trace_lines,
            files,
        }) => LogOptions {
            graph: args.contains(&LogArgument::Graph),
            color: args.contains(&LogArgument::Color),
            decorate: args.contains(&LogArgument::Decorate),
            show_header: args.contains(&LogArgument::ShowHeader),
            patch: args.contains(&LogArgument::Patch),
            stat: args.contains(&LogArgument::Stat),
            show_signature: args.contains(&LogArgument::ShowSignature),
            simplify_by_decoration: args.contains(&LogArgument::SimplifyByDecoration),
            limit,
            author,
            grep,
            search_changes,
            search_occurrences,
            trace_lines,
            files,
        },
        _ => LogOptions::default(),
    };
    let reflog = matches!(
        log_type,
        LogType::Reflog | LogType::ReflogOther(_) | LogType::Stashes
    );
    // Reflogs cannot be drawn as a graph (git rejects --graph with --walk-reflogs)
    // and, like Magit's reflog format, never show headers
    if reflog {
        options.graph = false;
        options.show_header = false;
    }
    match get_log_entries(&model.git_info.repository, &log_type, &options) {
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
                options,
            };

            // Dismiss the log popup
            model.popup = None;

            None
        }
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: format!("{}: {}", i18n::t().err_get_log_failed, e),
            });
            None
        }
    }
}
