use crate::{
    git::log::get_log_entries,
    i18n,
    model::{
        Line, LineContent, Model, PopupContent, ViewMode,
        arguments::{Arguments::LogArguments, DEFAULT_LOG_LIMIT, LogArgument},
    },
    msg::{LogType, Message},
};

pub fn update(model: &mut Model, log_type: LogType) -> Option<Message> {
    // Graph and refnames are shown by default; only disabled when toggled off
    // in the log popup
    let (
        graph,
        color,
        decorate,
        show_header,
        patch,
        stat,
        show_signature,
        limit,
        author,
        grep,
        search_changes,
        search_occurrences,
        trace_lines,
        simplify_by_decoration,
    ) = match model.arguments.take() {
        Some(LogArguments {
            args,
            limit,
            author,
            grep,
            search_changes,
            search_occurrences,
            trace_lines,
        }) => (
            args.contains(&LogArgument::Graph),
            args.contains(&LogArgument::Color),
            args.contains(&LogArgument::Decorate),
            args.contains(&LogArgument::ShowHeader),
            args.contains(&LogArgument::Patch),
            args.contains(&LogArgument::Stat),
            args.contains(&LogArgument::ShowSignature),
            limit,
            author,
            grep,
            search_changes,
            search_occurrences,
            trace_lines,
            args.contains(&LogArgument::SimplifyByDecoration),
        ),
        _ => (
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            DEFAULT_LOG_LIMIT,
            None,
            None,
            None,
            None,
            None,
            false,
        ),
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
        patch,
        stat,
        show_signature,
        limit,
        author.as_deref(),
        grep.as_deref(),
        search_changes.as_deref(),
        search_occurrences.as_deref(),
        trace_lines.as_deref(),
        simplify_by_decoration,
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
                patch,
                stat,
                show_signature,
                limit,
                author,
                grep,
                search_changes,
                search_occurrences,
                trace_lines,
                simplify_by_decoration,
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
