use crate::{
    git::log::get_log_entries,
    model::{Line, LineContent, Model, ViewMode},
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    match model.view_mode {
        ViewMode::Status => refresh_status(model),
        ViewMode::Log(log_type) => refresh_log(model, log_type),
    }
    None
}

fn refresh_status(model: &mut Model) {
    // Collect file paths that are currently collapsed (before refresh)
    // This handles files moving between unstaged and staged sections
    let collapsed_file_paths: std::collections::HashSet<String> = model
        .ui_model
        .collapsed_sections
        .iter()
        .filter_map(|section| section.file_path().map(String::from))
        .collect();

    // Collect all file paths that exist before the refresh (collapsed or expanded)
    // so we can distinguish new files from files the user explicitly expanded
    let existing_file_paths: std::collections::HashSet<String> = model
        .ui_model
        .lines
        .iter()
        .filter_map(|line| line.section.as_ref())
        .filter_map(|section| section.file_path().map(String::from))
        .collect();

    // Refresh the UI model by regenerating lines from git info
    if let Ok(lines) = model.git_info.get_lines() {
        model.ui_model.lines = lines;
        // Clamp cursor position if lines changed
        let max_pos = model.ui_model.lines.len().saturating_sub(1);
        if model.ui_model.cursor_position > max_pos {
            model.ui_model.cursor_position = max_pos;
        }

        // Restore collapsed state for files based on their paths
        // This preserves collapsed state when files move between staged/unstaged.
        // New files (paths not seen before the refresh) are collapsed by default.
        for line in &model.ui_model.lines {
            if let Some(section) = &line.section
                && let Some(path) = section.file_path()
            {
                if collapsed_file_paths.contains(path) {
                    model.ui_model.collapsed_sections.insert(section.clone());
                } else if !existing_file_paths.contains(path) && section.default_collapsed() {
                    model.ui_model.collapsed_sections.insert(section.clone());
                }
            }
        }

        // Clean up old file sections that no longer exist in the new lines
        let current_sections: std::collections::HashSet<_> = model
            .ui_model
            .lines
            .iter()
            .filter_map(|line| line.section.clone())
            .collect();
        model
            .ui_model
            .collapsed_sections
            .retain(|section| current_sections.contains(section) || section.file_path().is_none());
    }
}

fn refresh_log(model: &mut Model, log_type: crate::msg::LogType) {
    if let Ok(entries) = get_log_entries(&model.git_info.repository, log_type) {
        let lines: Vec<Line> = entries
            .into_iter()
            .map(|entry| Line {
                content: LineContent::LogLine(entry),
                section: None,
            })
            .collect();

        model.ui_model.lines = lines;

        // Clamp cursor position if lines changed
        let max_pos = model.ui_model.lines.len().saturating_sub(1);
        if model.ui_model.cursor_position > max_pos {
            model.ui_model.cursor_position = max_pos;
        }
    }
}
