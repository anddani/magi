use std::time::{Duration, Instant};

use crate::{
    git::commit::{self, CommitResult},
    model::{DialogContent, LineContent, Model, RunningState, Toast, ToastStyle},
    msg::{util::visible_lines_between, Message},
};

/// Duration for toast notifications
pub const TOAST_DURATION: Duration = Duration::from_secs(5);

/// Processes a [`Message`], modifying the passed model.
///
/// Returns a follow up [`Message`] for sequences of actions.
/// e.g. after a stage, a [`Message::Refresh`] should be triggered.
pub fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::Quit => model.running_state = RunningState::Done,
        Message::Refresh => {
            // Collect file paths that are currently collapsed (before refresh)
            // This handles files moving between unstaged and staged sections
            let collapsed_file_paths: std::collections::HashSet<String> = model
                .ui_model
                .collapsed_sections
                .iter()
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
                // This preserves collapsed state when files move between staged/unstaged
                for line in &model.ui_model.lines {
                    if let Some(section) = &line.section {
                        if let Some(path) = section.file_path() {
                            if collapsed_file_paths.contains(path) {
                                model.ui_model.collapsed_sections.insert(section.clone());
                            }
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
                model.ui_model.collapsed_sections.retain(|section| {
                    current_sections.contains(section) || section.file_path().is_none()
                });
            }
        }
        Message::MoveUp => {
            // Find the previous visible line
            let mut new_pos = model.ui_model.cursor_position;
            while new_pos > 0 {
                new_pos -= 1;
                if !model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections) {
                    model.ui_model.cursor_position = new_pos;
                    // Scroll up if cursor moves above viewport
                    if model.ui_model.cursor_position < model.ui_model.scroll_offset {
                        model.ui_model.scroll_offset = model.ui_model.cursor_position;
                    }
                    break;
                }
            }
        }
        Message::MoveDown => {
            let max_pos = model.ui_model.lines.len().saturating_sub(1);
            // Find the next visible line
            let mut new_pos = model.ui_model.cursor_position;
            while new_pos < max_pos {
                new_pos += 1;
                if !model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections) {
                    model.ui_model.cursor_position = new_pos;
                    // Scroll down if cursor moves below viewport
                    let viewport_height = model.ui_model.viewport_height;
                    if viewport_height > 0 {
                        // Count visible lines from scroll_offset to cursor (exclusive)
                        let visible_before_cursor = visible_lines_between(
                            &model.ui_model.lines,
                            model.ui_model.scroll_offset,
                            model.ui_model.cursor_position,
                            &model.ui_model.collapsed_sections,
                        );
                        // If visible lines before cursor >= viewport_height, cursor is below viewport
                        if visible_before_cursor >= viewport_height {
                            // Scroll so cursor is at bottom of viewport
                            // Walk back from cursor to find scroll position with (viewport_height - 1)
                            // visible lines before cursor
                            let mut new_scroll = model.ui_model.cursor_position;
                            let mut visible_count = 0;
                            while new_scroll > 0 && visible_count < viewport_height - 1 {
                                new_scroll -= 1;
                                if !model.ui_model.lines[new_scroll]
                                    .is_hidden(&model.ui_model.collapsed_sections)
                                {
                                    visible_count += 1;
                                }
                            }
                            model.ui_model.scroll_offset = new_scroll;
                        }
                    }
                    break;
                }
            }
        }
        Message::ToggleSection => {
            // Get the current line and check if it's a collapsible header
            if let Some(line) = model.ui_model.lines.get(model.ui_model.cursor_position) {
                if let Some(section) = line.collapsible_section() {
                    // Toggle the section in collapsed_sections
                    if model.ui_model.collapsed_sections.contains(&section) {
                        model.ui_model.collapsed_sections.remove(&section);
                    } else {
                        model.ui_model.collapsed_sections.insert(section);
                    }
                }
            }
        }
        Message::HalfPageUp => {
            let half_page = model.ui_model.viewport_height / 2;
            // Move up by half_page visible lines
            let mut visible_count = 0;
            let mut new_pos = model.ui_model.cursor_position;
            while new_pos > 0 && visible_count < half_page {
                new_pos -= 1;
                if !model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections) {
                    visible_count += 1;
                }
            }
            // Make sure we land on a visible line (try backward first, then forward)
            while new_pos > 0
                && model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections)
            {
                new_pos -= 1;
            }
            // If still on a hidden line (reached beginning), search forward
            let max_pos = model.ui_model.lines.len().saturating_sub(1);
            while new_pos < max_pos
                && model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections)
            {
                new_pos += 1;
            }
            model.ui_model.cursor_position = new_pos;
            // Scroll up if cursor moves above viewport
            if model.ui_model.cursor_position < model.ui_model.scroll_offset {
                model.ui_model.scroll_offset = model.ui_model.cursor_position;
            }
        }
        Message::HalfPageDown => {
            let half_page = model.ui_model.viewport_height / 2;
            let max_pos = model.ui_model.lines.len().saturating_sub(1);
            // Move down by half_page visible lines
            let mut visible_count = 0;
            let mut new_pos = model.ui_model.cursor_position;
            while new_pos < max_pos && visible_count < half_page {
                new_pos += 1;
                if !model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections) {
                    visible_count += 1;
                }
            }
            // Make sure we land on a visible line (try forward first, then backward)
            while new_pos < max_pos
                && model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections)
            {
                new_pos += 1;
            }
            // If still on a hidden line (reached end), search backward
            while new_pos > 0
                && model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections)
            {
                new_pos -= 1;
            }
            model.ui_model.cursor_position = new_pos;
            // Scroll down if cursor moves below viewport
            let viewport_height = model.ui_model.viewport_height;
            if viewport_height > 0 {
                let visible_before_cursor = visible_lines_between(
                    &model.ui_model.lines,
                    model.ui_model.scroll_offset,
                    model.ui_model.cursor_position,
                    &model.ui_model.collapsed_sections,
                );
                if visible_before_cursor >= viewport_height {
                    let mut new_scroll = model.ui_model.cursor_position;
                    let mut scroll_visible_count = 0;
                    while new_scroll > 0 && scroll_visible_count < viewport_height - 1 {
                        new_scroll -= 1;
                        if !model.ui_model.lines[new_scroll]
                            .is_hidden(&model.ui_model.collapsed_sections)
                        {
                            scroll_visible_count += 1;
                        }
                    }
                    model.ui_model.scroll_offset = new_scroll;
                }
            }
        }
        Message::ScrollLineDown => {
            // Scroll viewport down by one visible line (C-e in Vim)
            let max_pos = model.ui_model.lines.len().saturating_sub(1);
            let viewport_height = model.ui_model.viewport_height;
            if viewport_height == 0 {
                return None;
            }

            // Find the next visible line after current scroll_offset
            let mut new_scroll = model.ui_model.scroll_offset;
            while new_scroll < max_pos {
                new_scroll += 1;
                if !model.ui_model.lines[new_scroll].is_hidden(&model.ui_model.collapsed_sections) {
                    break;
                }
            }

            // Only scroll if there's content below to show
            if new_scroll <= max_pos
                && !model.ui_model.lines[new_scroll].is_hidden(&model.ui_model.collapsed_sections)
            {
                model.ui_model.scroll_offset = new_scroll;

                // If cursor is now above viewport, move it to the top visible line
                if model.ui_model.cursor_position < model.ui_model.scroll_offset {
                    model.ui_model.cursor_position = model.ui_model.scroll_offset;
                }
            }
        }
        Message::ScrollLineUp => {
            // Scroll viewport up by one visible line (C-y in Vim)
            if model.ui_model.scroll_offset == 0 {
                return None;
            }

            let viewport_height = model.ui_model.viewport_height;
            if viewport_height == 0 {
                return None;
            }

            // Find the previous visible line before current scroll_offset
            let mut new_scroll = model.ui_model.scroll_offset;
            while new_scroll > 0 {
                new_scroll -= 1;
                if !model.ui_model.lines[new_scroll].is_hidden(&model.ui_model.collapsed_sections) {
                    break;
                }
            }

            model.ui_model.scroll_offset = new_scroll;

            // If cursor is now below viewport, move it to the bottom visible line
            let visible_before_cursor = visible_lines_between(
                &model.ui_model.lines,
                model.ui_model.scroll_offset,
                model.ui_model.cursor_position,
                &model.ui_model.collapsed_sections,
            );

            if visible_before_cursor >= viewport_height {
                // Find the last visible line in the viewport
                let mut new_cursor = model.ui_model.scroll_offset;
                let mut visible_count = 0;
                let max_pos = model.ui_model.lines.len().saturating_sub(1);
                while new_cursor < max_pos && visible_count < viewport_height - 1 {
                    new_cursor += 1;
                    if !model.ui_model.lines[new_cursor]
                        .is_hidden(&model.ui_model.collapsed_sections)
                    {
                        visible_count += 1;
                    }
                }
                // Make sure we're on a visible line
                while new_cursor > 0
                    && model.ui_model.lines[new_cursor]
                        .is_hidden(&model.ui_model.collapsed_sections)
                {
                    new_cursor -= 1;
                }
                model.ui_model.cursor_position = new_cursor;
            }
        }
        Message::Commit => {
            if let Ok(false) = model.git_info.has_staged_changes() {
                model.toast = Some(Toast {
                    message: "Nothing staged to commit".to_string(),
                    style: ToastStyle::Warning,
                    expires_at: Instant::now() + TOAST_DURATION,
                });
                return None;
            }

            if let Some(repo_path) = model.git_info.repository.workdir() {
                match commit::run_commit_with_editor(repo_path) {
                    Ok(CommitResult { success, message }) => {
                        model.toast = Some(Toast {
                            message,
                            style: if success {
                                ToastStyle::Success
                            } else {
                                ToastStyle::Warning
                            },
                            expires_at: Instant::now() + TOAST_DURATION,
                        });
                    }
                    Err(e) => {
                        model.dialog = Some(DialogContent::Error {
                            message: e.to_string(),
                        });
                    }
                }
            } else {
                model.dialog = Some(DialogContent::Error {
                    message: "Repository working directory not found".into(),
                });
            };
            return Some(Message::Refresh);
        }
        Message::DismissDialog => {
            model.dialog = None;
        }
        Message::StageAllModified => {
            let repo_path = model.git_info.repository.workdir()?;
            let files: Vec<&str> = model
                .ui_model
                .lines
                .iter()
                .filter_map(|line| match &line.content {
                    LineContent::UnstagedFile(fc) => Some(fc.path.as_str()),
                    _ => None,
                })
                .collect();
            if let Err(e) = crate::git::stage::stage_files(repo_path, &files) {
                model.dialog = Some(DialogContent::Error {
                    message: format!("Error staging files: {}", e),
                });
            }
            return Some(Message::Refresh);
        }
        Message::UnstageAll => {
            let repo_path = model.git_info.repository.workdir()?;
            let files: Vec<&str> = model
                .ui_model
                .lines
                .iter()
                .filter_map(|line| match &line.content {
                    LineContent::StagedFile(fc) => Some(fc.path.as_str()),
                    _ => None,
                })
                .collect();
            if let Err(e) = crate::git::stage::unstage_files(repo_path, &files) {
                model.dialog = Some(DialogContent::Error {
                    message: format!("Error unstaging files: {}", e),
                });
            }
            return Some(Message::Refresh);
        }
    };
    None
}
