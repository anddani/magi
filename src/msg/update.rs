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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::config::Theme;
    use crate::git::GitInfo;
    use crate::model::{Line, LineContent, RunningState, SectionType, UiModel};

    #[test]
    fn test_refresh_message() -> Result<(), git2::Error> {
        // Create a test model with git info
        let git_info = GitInfo::new()?;
        let initial_lines = git_info.get_lines().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: initial_lines.clone(),
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Clear the lines to simulate outdated state
        model.ui_model.lines.clear();
        assert!(model.ui_model.lines.is_empty());

        // Send refresh message
        update(&mut model, Message::Refresh);

        // Verify that lines were refreshed
        assert!(!model.ui_model.lines.is_empty());
        assert_eq!(model.running_state, RunningState::Running);

        Ok(())
    }

    #[test]
    fn test_quit_message() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel::default(),
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Send quit message
        update(&mut model, Message::Quit);

        // Verify that running state changed to Done
        assert_eq!(model.running_state, RunningState::Done);
    }

    fn create_test_lines(count: usize) -> Vec<Line> {
        (0..count)
            .map(|_| Line {
                content: LineContent::EmptyLine,
                section: None,
            })
            .collect()
    }

    #[test]
    fn test_move_down() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(5),
                cursor_position: 0,
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        update(&mut model, Message::MoveDown);
        assert_eq!(model.ui_model.cursor_position, 1);

        update(&mut model, Message::MoveDown);
        assert_eq!(model.ui_model.cursor_position, 2);
    }

    #[test]
    fn test_move_up() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(5),
                cursor_position: 2,
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        update(&mut model, Message::MoveUp);
        assert_eq!(model.ui_model.cursor_position, 1);

        update(&mut model, Message::MoveUp);
        assert_eq!(model.ui_model.cursor_position, 0);
    }

    #[test]
    fn test_move_up_at_top() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(5),
                cursor_position: 0,
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        update(&mut model, Message::MoveUp);
        assert_eq!(model.ui_model.cursor_position, 0);
    }

    #[test]
    fn test_move_down_at_bottom() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(5),
                cursor_position: 4,
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        update(&mut model, Message::MoveDown);
        assert_eq!(model.ui_model.cursor_position, 4);
    }

    #[test]
    fn test_scroll_down_when_cursor_leaves_viewport() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(10),
                cursor_position: 2,
                scroll_offset: 0,
                viewport_height: 3,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Move to position 3, which is outside viewport (0-2)
        update(&mut model, Message::MoveDown);
        assert_eq!(model.ui_model.cursor_position, 3);
        assert_eq!(model.ui_model.scroll_offset, 1);
    }

    #[test]
    fn test_scroll_up_when_cursor_leaves_viewport() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(10),
                cursor_position: 5,
                scroll_offset: 5,
                viewport_height: 3,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Move to position 4, which is above scroll_offset
        update(&mut model, Message::MoveUp);
        assert_eq!(model.ui_model.cursor_position, 4);
        assert_eq!(model.ui_model.scroll_offset, 4);
    }

    #[test]
    fn test_half_page_down() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(20),
                cursor_position: 0,
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        update(&mut model, Message::HalfPageDown);
        assert_eq!(model.ui_model.cursor_position, 5); // half of 10
    }

    #[test]
    fn test_half_page_up() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(20),
                cursor_position: 10,
                scroll_offset: 5,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        update(&mut model, Message::HalfPageUp);
        assert_eq!(model.ui_model.cursor_position, 5); // 10 - 5
    }

    #[test]
    fn test_half_page_up_at_top() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(20),
                cursor_position: 0,
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        update(&mut model, Message::HalfPageUp);
        assert_eq!(model.ui_model.cursor_position, 0); // stays at 0
        assert_eq!(model.ui_model.scroll_offset, 0);
    }

    #[test]
    fn test_half_page_down_at_bottom() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(20),
                cursor_position: 19,
                scroll_offset: 10,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        update(&mut model, Message::HalfPageDown);
        assert_eq!(model.ui_model.cursor_position, 19); // stays at max
    }

    #[test]
    fn test_half_page_down_clamps_to_max() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(20),
                cursor_position: 17,
                scroll_offset: 10,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // 17 + 5 = 22, but max is 19
        update(&mut model, Message::HalfPageDown);
        assert_eq!(model.ui_model.cursor_position, 19);
    }

    #[test]
    fn test_half_page_up_clamps_to_zero() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(20),
                cursor_position: 2,
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // 2 - 5 would be negative, should clamp to 0
        update(&mut model, Message::HalfPageUp);
        assert_eq!(model.ui_model.cursor_position, 0);
    }

    #[test]
    fn test_half_page_down_scrolls_viewport() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(30),
                cursor_position: 8,
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Cursor at 8, move down 5 -> 13, which is outside viewport (0-9)
        update(&mut model, Message::HalfPageDown);
        assert_eq!(model.ui_model.cursor_position, 13);
        assert_eq!(model.ui_model.scroll_offset, 4); // 13 - 10 + 1
    }

    #[test]
    fn test_half_page_up_scrolls_viewport() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(30),
                cursor_position: 12,
                scroll_offset: 10,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Cursor at 12, scroll at 10, move up 5 -> 7, which is above scroll_offset
        update(&mut model, Message::HalfPageUp);
        assert_eq!(model.ui_model.cursor_position, 7);
        assert_eq!(model.ui_model.scroll_offset, 7);
    }

    #[test]
    fn test_half_page_with_small_viewport() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(10),
                cursor_position: 5,
                scroll_offset: 3,
                viewport_height: 2,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Half of 2 is 1
        update(&mut model, Message::HalfPageDown);
        assert_eq!(model.ui_model.cursor_position, 6);

        update(&mut model, Message::HalfPageUp);
        assert_eq!(model.ui_model.cursor_position, 5);
    }

    #[test]
    fn test_half_page_with_zero_viewport() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(10),
                cursor_position: 5,
                scroll_offset: 0,
                viewport_height: 0,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Half of 0 is 0, cursor shouldn't move
        update(&mut model, Message::HalfPageDown);
        assert_eq!(model.ui_model.cursor_position, 5);

        update(&mut model, Message::HalfPageUp);
        assert_eq!(model.ui_model.cursor_position, 5);
    }

    #[test]
    fn test_half_page_with_empty_lines() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: vec![],
                cursor_position: 0,
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // With no lines, cursor should stay at 0
        update(&mut model, Message::HalfPageDown);
        assert_eq!(model.ui_model.cursor_position, 0);

        update(&mut model, Message::HalfPageUp);
        assert_eq!(model.ui_model.cursor_position, 0);
    }

    fn create_section_lines() -> Vec<Line> {
        use crate::model::{FileChange, FileStatus, SectionType};

        vec![
            // 0: Section header
            Line {
                content: LineContent::SectionHeader {
                    title: "Untracked files".to_string(),
                    count: Some(2),
                },
                section: Some(SectionType::UntrackedFiles),
            },
            // 1: Untracked file
            Line {
                content: LineContent::UntrackedFile("a.txt".to_string()),
                section: Some(SectionType::UntrackedFiles),
            },
            // 2: Untracked file
            Line {
                content: LineContent::UntrackedFile("b.txt".to_string()),
                section: Some(SectionType::UntrackedFiles),
            },
            // 3: Empty line
            Line {
                content: LineContent::EmptyLine,
                section: None,
            },
            // 4: Section header
            Line {
                content: LineContent::SectionHeader {
                    title: "Unstaged changes".to_string(),
                    count: Some(1),
                },
                section: Some(SectionType::UnstagedChanges),
            },
            // 5: File
            Line {
                content: LineContent::UnstagedFile(FileChange {
                    path: "foo.rs".to_string(),
                    status: FileStatus::Modified,
                }),
                section: Some(SectionType::UnstagedFile {
                    path: "foo.rs".to_string(),
                }),
            },
            // 6: Hunk (would be hidden when file is collapsed)
            Line {
                content: LineContent::EmptyLine, // Simplified for testing
                section: Some(SectionType::UnstagedHunk {
                    path: "foo.rs".to_string(),
                    hunk_index: 0,
                }),
            },
        ]
    }

    #[test]
    fn test_toggle_section_on_header() {
        use crate::model::SectionType;

        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_section_lines(),
                cursor_position: 0, // On section header
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Initially not collapsed
        assert!(!model
            .ui_model
            .collapsed_sections
            .contains(&SectionType::UntrackedFiles));

        // Toggle should collapse
        update(&mut model, Message::ToggleSection);
        assert!(model
            .ui_model
            .collapsed_sections
            .contains(&SectionType::UntrackedFiles));

        // Toggle again should expand
        update(&mut model, Message::ToggleSection);
        assert!(!model
            .ui_model
            .collapsed_sections
            .contains(&SectionType::UntrackedFiles));
    }

    #[test]
    fn test_toggle_section_on_file() {
        use crate::model::SectionType;

        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_section_lines(),
                cursor_position: 5, // On file (foo.rs)
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        let file_section = SectionType::UnstagedFile {
            path: "foo.rs".to_string(),
        };

        // Initially not collapsed
        assert!(!model.ui_model.collapsed_sections.contains(&file_section));

        // Toggle should collapse the file's section
        update(&mut model, Message::ToggleSection);
        assert!(model.ui_model.collapsed_sections.contains(&file_section));
    }

    #[test]
    fn test_toggle_section_on_non_header_does_nothing() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_section_lines(),
                cursor_position: 1, // On untracked file (not a collapsible header)
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Toggle on non-header should do nothing
        update(&mut model, Message::ToggleSection);
        assert!(model.ui_model.collapsed_sections.is_empty());
    }

    #[test]
    fn test_move_down_skips_hidden_lines() {
        use crate::model::SectionType;

        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_section_lines(),
                cursor_position: 0, // On "Untracked files" header
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Collapse the Untracked files section
        model
            .ui_model
            .collapsed_sections
            .insert(SectionType::UntrackedFiles);

        // Move down should skip hidden lines (1, 2) and land on empty line (3)
        update(&mut model, Message::MoveDown);
        assert_eq!(model.ui_model.cursor_position, 3);
    }

    #[test]
    fn test_move_up_skips_hidden_lines() {
        use crate::model::SectionType;

        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_section_lines(),
                cursor_position: 3, // On empty line
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Collapse the Untracked files section
        model
            .ui_model
            .collapsed_sections
            .insert(SectionType::UntrackedFiles);

        // Move up should skip hidden lines (2, 1) and land on header (0)
        update(&mut model, Message::MoveUp);
        assert_eq!(model.ui_model.cursor_position, 0);
    }

    /// Create lines simulating two files with many diff lines each
    fn create_two_file_lines() -> Vec<Line> {
        use crate::model::{DiffHunk, DiffLine, DiffLineType, FileChange, FileStatus};

        let mut lines = Vec::new();

        // 0: Section header
        lines.push(Line {
            content: LineContent::SectionHeader {
                title: "Unstaged changes".to_string(),
                count: Some(2),
            },
            section: Some(SectionType::UnstagedChanges),
        });

        // 1: First file header
        lines.push(Line {
            content: LineContent::UnstagedFile(FileChange {
                path: "file1.rs".to_string(),
                status: FileStatus::Modified,
            }),
            section: Some(SectionType::UnstagedFile {
                path: "file1.rs".to_string(),
            }),
        });

        // 2: First file hunk header
        lines.push(Line {
            content: LineContent::DiffHunk(DiffHunk {
                header: "@@ -1,20 +1,25 @@".to_string(),
            }),
            section: Some(SectionType::UnstagedHunk {
                path: "file1.rs".to_string(),
                hunk_index: 0,
            }),
        });

        // 3-22: First file diff lines (20 lines)
        for i in 0..20 {
            lines.push(Line {
                content: LineContent::DiffLine(DiffLine {
                    content: format!(" context line {}", i),
                    line_type: DiffLineType::Context,
                }),
                section: Some(SectionType::UnstagedHunk {
                    path: "file1.rs".to_string(),
                    hunk_index: 0,
                }),
            });
        }

        // 23: Second file header
        lines.push(Line {
            content: LineContent::UnstagedFile(FileChange {
                path: "file2.rs".to_string(),
                status: FileStatus::Modified,
            }),
            section: Some(SectionType::UnstagedFile {
                path: "file2.rs".to_string(),
            }),
        });

        // 24: Second file hunk header
        lines.push(Line {
            content: LineContent::DiffHunk(DiffHunk {
                header: "@@ -1,10 +1,15 @@".to_string(),
            }),
            section: Some(SectionType::UnstagedHunk {
                path: "file2.rs".to_string(),
                hunk_index: 0,
            }),
        });

        // 25-34: Second file diff lines (10 lines)
        for i in 0..10 {
            lines.push(Line {
                content: LineContent::DiffLine(DiffLine {
                    content: format!(" context line {}", i),
                    line_type: DiffLineType::Context,
                }),
                section: Some(SectionType::UnstagedHunk {
                    path: "file2.rs".to_string(),
                    hunk_index: 0,
                }),
            });
        }

        lines
    }

    #[test]
    fn test_scroll_with_collapsed_file_does_not_over_scroll() {
        // This tests the bug where navigating from a collapsed file to the next file
        // caused the screen to scroll so the target file was at the top instead of bottom
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_two_file_lines(),
                cursor_position: 1, // On first file header (file1.rs)
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Collapse the first file - this hides lines 2-22 (hunk + 20 diff lines)
        model
            .ui_model
            .collapsed_sections
            .insert(SectionType::UnstagedFile {
                path: "file1.rs".to_string(),
            });

        // Move down should go to the second file header (line 23)
        update(&mut model, Message::MoveDown);
        assert_eq!(model.ui_model.cursor_position, 23);

        // With viewport_height=10, and only 3 visible lines before cursor
        // (line 0: section header, line 1: file1 header, line 23: file2 header)
        // the scroll_offset should NOT change since cursor is still in viewport
        // Visible lines from scroll_offset=0: 0, 1, 23 = only 3 lines before position 23
        // 3 < 10, so no scroll needed
        assert_eq!(
            model.ui_model.scroll_offset, 0,
            "scroll_offset should remain 0 since cursor is within viewport"
        );
    }

    #[test]
    fn test_visible_lines_between() {
        let lines = create_section_lines();
        let mut collapsed = HashSet::new();

        // No collapsed sections: all lines visible
        assert_eq!(visible_lines_between(&lines, 0, 3, &collapsed), 3);
        assert_eq!(visible_lines_between(&lines, 0, 7, &collapsed), 7);

        // Collapse UntrackedFiles section - hides lines 1, 2
        collapsed.insert(SectionType::UntrackedFiles);
        // Lines 0-3: line 0 visible, lines 1-2 hidden, so only 1 visible
        assert_eq!(visible_lines_between(&lines, 0, 3, &collapsed), 1);
        // Lines 0-7: lines 0, 3, 4, 5, 6 visible = 5
        assert_eq!(visible_lines_between(&lines, 0, 7, &collapsed), 5);
    }

    /// Create lines where both files are collapsed, leaving few visible lines
    fn create_both_files_collapsed_lines() -> Vec<Line> {
        use crate::model::{DiffLine, DiffLineType, FileChange, FileStatus};

        let mut lines = Vec::new();

        // 0: Section header
        lines.push(Line {
            content: LineContent::SectionHeader {
                title: "Unstaged changes".to_string(),
                count: Some(2),
            },
            section: Some(SectionType::UnstagedChanges),
        });

        // 1: First file header
        lines.push(Line {
            content: LineContent::UnstagedFile(FileChange {
                path: "file1.rs".to_string(),
                status: FileStatus::Modified,
            }),
            section: Some(SectionType::UnstagedFile {
                path: "file1.rs".to_string(),
            }),
        });

        // 2-11: First file hunks (10 lines, will be hidden when collapsed)
        for i in 0..10 {
            lines.push(Line {
                content: LineContent::DiffLine(DiffLine {
                    content: format!(" line {}", i),
                    line_type: DiffLineType::Context,
                }),
                section: Some(SectionType::UnstagedHunk {
                    path: "file1.rs".to_string(),
                    hunk_index: 0,
                }),
            });
        }

        // 12: Second file header
        lines.push(Line {
            content: LineContent::UnstagedFile(FileChange {
                path: "file2.rs".to_string(),
                status: FileStatus::Modified,
            }),
            section: Some(SectionType::UnstagedFile {
                path: "file2.rs".to_string(),
            }),
        });

        // 13-22: Second file hunks (10 lines, will be hidden when collapsed)
        for i in 0..10 {
            lines.push(Line {
                content: LineContent::DiffLine(DiffLine {
                    content: format!(" line {}", i),
                    line_type: DiffLineType::Context,
                }),
                section: Some(SectionType::UnstagedHunk {
                    path: "file2.rs".to_string(),
                    hunk_index: 0,
                }),
            });
        }

        lines // Total: 23 lines (indices 0-22)
    }

    #[test]
    fn test_half_page_down_with_collapsed_sections() {
        // This tests the bug where Ctrl+d with collapsed sections
        // causes cursor to land on a hidden line
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_both_files_collapsed_lines(),
                cursor_position: 1, // On first file header
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Collapse both files
        model
            .ui_model
            .collapsed_sections
            .insert(SectionType::UnstagedFile {
                path: "file1.rs".to_string(),
            });
        model
            .ui_model
            .collapsed_sections
            .insert(SectionType::UnstagedFile {
                path: "file2.rs".to_string(),
            });

        // Visible lines are: 0 (header), 1 (file1), 12 (file2)
        // half_page = 5, but only 2 visible lines after cursor
        // Cursor should land on line 12 (file2), the last visible line after cursor

        update(&mut model, Message::HalfPageDown);

        // Cursor must be on a visible line
        assert!(
            !model.ui_model.lines[model.ui_model.cursor_position]
                .is_hidden(&model.ui_model.collapsed_sections),
            "Cursor should be on a visible line, but landed on hidden line at position {}",
            model.ui_model.cursor_position
        );

        // Should land on file2 header (line 12)
        assert_eq!(
            model.ui_model.cursor_position, 12,
            "Cursor should land on file2 header"
        );
    }

    #[test]
    fn test_half_page_up_with_collapsed_sections() {
        // Same test but for HalfPageUp
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_both_files_collapsed_lines(),
                cursor_position: 12, // On second file header
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Collapse both files
        model
            .ui_model
            .collapsed_sections
            .insert(SectionType::UnstagedFile {
                path: "file1.rs".to_string(),
            });
        model
            .ui_model
            .collapsed_sections
            .insert(SectionType::UnstagedFile {
                path: "file2.rs".to_string(),
            });

        // Visible lines are: 0 (header), 1 (file1), 12 (file2)
        // half_page = 5, but only 2 visible lines before cursor

        update(&mut model, Message::HalfPageUp);

        // Cursor must be on a visible line
        assert!(
            !model.ui_model.lines[model.ui_model.cursor_position]
                .is_hidden(&model.ui_model.collapsed_sections),
            "Cursor should be on a visible line, but landed on hidden line at position {}",
            model.ui_model.cursor_position
        );
    }

    #[test]
    fn test_commit_without_staged_changes_shows_error_popup() {
        // This test verifies that trying to commit without staged changes
        // shows a toast instead of launching the editor
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel::default(),
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Send commit message (assuming no staged changes in the test repo)
        update(&mut model, Message::Commit);

        // If there are no staged changes, toast should show a warning
        // If there are staged changes, running_state should be LaunchExternalCommand
        // Either outcome is valid depending on repo state, but one must happen
        let has_toast = model.toast.is_some();
        let wants_editor =
            model.running_state == RunningState::LaunchExternalCommand(Message::Commit);
        assert!(
            has_toast || wants_editor,
            "Commit should either show toast or trigger editor launch"
        );
    }

    #[test]
    fn test_dismiss_dialog_clears_dialog() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel::default(),
            theme: Theme::default(),
            dialog: Some(DialogContent::Error {
                message: "Test error".to_string(),
            }),
            toast: None,
        };

        // Dialog should be present
        assert!(model.dialog.is_some());

        // Send dismiss message
        update(&mut model, Message::DismissDialog);

        // Dialog should be cleared
        assert!(model.dialog.is_none());
    }

    #[test]
    fn test_scroll_line_down() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(20),
                cursor_position: 0,
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Scroll down once
        update(&mut model, Message::ScrollLineDown);
        assert_eq!(model.ui_model.scroll_offset, 1);
        assert_eq!(model.ui_model.cursor_position, 1); // Cursor moves with viewport

        // Scroll down again
        update(&mut model, Message::ScrollLineDown);
        assert_eq!(model.ui_model.scroll_offset, 2);
        assert_eq!(model.ui_model.cursor_position, 2);
    }

    #[test]
    fn test_scroll_line_up() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(20),
                cursor_position: 15,
                scroll_offset: 10,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Scroll up once
        update(&mut model, Message::ScrollLineUp);
        assert_eq!(model.ui_model.scroll_offset, 9);
        assert_eq!(model.ui_model.cursor_position, 15); // Cursor stays in place

        // Scroll up more times until cursor would leave viewport
        for _ in 0..6 {
            update(&mut model, Message::ScrollLineUp);
        }
        // scroll_offset should be 3, cursor should move to bottom of viewport
        assert_eq!(model.ui_model.scroll_offset, 3);
        // Cursor should be at bottom of viewport (scroll_offset + viewport_height - 1 = 3 + 10 - 1 = 12)
        assert_eq!(model.ui_model.cursor_position, 12);
    }

    #[test]
    fn test_scroll_line_down_cursor_follows_top() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(20),
                cursor_position: 0,
                scroll_offset: 0,
                viewport_height: 5,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Cursor at top of viewport, scroll down - cursor should follow
        update(&mut model, Message::ScrollLineDown);
        assert_eq!(model.ui_model.scroll_offset, 1);
        assert_eq!(model.ui_model.cursor_position, 1);
    }

    #[test]
    fn test_scroll_line_up_cursor_follows_bottom() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(20),
                cursor_position: 14, // At bottom of viewport (10 + 5 - 1)
                scroll_offset: 10,
                viewport_height: 5,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Cursor at bottom of viewport, scroll up - cursor should stay in viewport
        update(&mut model, Message::ScrollLineUp);
        assert_eq!(model.ui_model.scroll_offset, 9);
        assert_eq!(model.ui_model.cursor_position, 13); // Follows bottom of viewport
    }

    #[test]
    fn test_scroll_line_down_at_end() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(10),
                cursor_position: 9,
                scroll_offset: 5, // Already scrolled down
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Try to scroll past end
        update(&mut model, Message::ScrollLineDown);
        update(&mut model, Message::ScrollLineDown);
        update(&mut model, Message::ScrollLineDown);
        update(&mut model, Message::ScrollLineDown);
        // Should stop at max_pos (9) since viewport can't go beyond content
        assert!(model.ui_model.scroll_offset <= 9);
    }

    #[test]
    fn test_scroll_line_up_at_start() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(20),
                cursor_position: 0,
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Try to scroll up at top - should have no effect
        update(&mut model, Message::ScrollLineUp);
        assert_eq!(model.ui_model.scroll_offset, 0);
        assert_eq!(model.ui_model.cursor_position, 0);
    }

    #[test]
    fn test_scroll_line_down_with_collapsed_sections() {
        use crate::model::SectionType;

        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_section_lines(),
                cursor_position: 0,
                scroll_offset: 0,
                viewport_height: 10,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Collapse untracked files (hides lines 1, 2)
        model
            .ui_model
            .collapsed_sections
            .insert(SectionType::UntrackedFiles);

        // Scroll down should skip hidden lines
        update(&mut model, Message::ScrollLineDown);
        // Should land on line 3 (empty line) which is the next visible line
        assert_eq!(model.ui_model.scroll_offset, 3);
        assert_eq!(model.ui_model.cursor_position, 3);
    }

    #[test]
    fn test_scroll_with_zero_viewport() {
        let git_info = GitInfo::new().unwrap();
        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines: create_test_lines(10),
                cursor_position: 5,
                scroll_offset: 3,
                viewport_height: 0,
                ..Default::default()
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // With zero viewport, scrolling should do nothing
        let original_scroll = model.ui_model.scroll_offset;
        let original_cursor = model.ui_model.cursor_position;

        update(&mut model, Message::ScrollLineDown);
        assert_eq!(model.ui_model.scroll_offset, original_scroll);
        assert_eq!(model.ui_model.cursor_position, original_cursor);

        update(&mut model, Message::ScrollLineUp);
        assert_eq!(model.ui_model.scroll_offset, original_scroll);
        assert_eq!(model.ui_model.cursor_position, original_cursor);
    }

    #[test]
    fn test_collapsed_state_preserved_when_staging_all() {
        use crate::git::test_repo::TestRepo;
        use std::fs;

        // Create a test repo with a modified file
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();

        // Modify the tracked file to create unstaged changes
        let file_path = repo_path.join("test.txt");
        fs::write(&file_path, "modified content").unwrap();

        // Create GitInfo from test repo
        let git_info = GitInfo::new_from_path(repo_path).unwrap();
        let lines = git_info.get_lines().unwrap();

        // Find the unstaged file section and collapse it
        let mut collapsed_sections = HashSet::new();
        for line in &lines {
            if let Some(section) = &line.section {
                if let SectionType::UnstagedFile { path } = section {
                    if path == "test.txt" {
                        collapsed_sections.insert(section.clone());
                    }
                }
            }
        }

        // Verify we found and collapsed the file
        assert!(
            collapsed_sections.contains(&SectionType::UnstagedFile {
                path: "test.txt".to_string()
            }),
            "Should have found and collapsed the unstaged file"
        );

        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines,
                cursor_position: 0,
                scroll_offset: 0,
                viewport_height: 20,
                collapsed_sections,
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Stage all modified files
        let follow_up = update(&mut model, Message::StageAllModified);
        assert_eq!(follow_up, Some(Message::Refresh));

        // Process the refresh
        update(&mut model, Message::Refresh);

        // Verify the file is now in staged section and still collapsed
        assert!(
            model
                .ui_model
                .collapsed_sections
                .contains(&SectionType::StagedFile {
                    path: "test.txt".to_string()
                }),
            "Staged file should be collapsed after StageAllModified"
        );

        // The old unstaged file section should be cleaned up
        assert!(
            !model
                .ui_model
                .collapsed_sections
                .contains(&SectionType::UnstagedFile {
                    path: "test.txt".to_string()
                }),
            "Old unstaged file section should be cleaned up"
        );
    }

    #[test]
    fn test_collapsed_state_preserved_when_unstaging_all() {
        use crate::git::test_repo::TestRepo;
        use std::fs;

        // Create a test repo
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();

        // Modify and stage the file
        let file_path = repo_path.join("test.txt");
        fs::write(&file_path, "modified content").unwrap();
        crate::git::stage::stage_files(repo_path, &["test.txt"]).unwrap();

        // Create GitInfo from test repo
        let git_info = GitInfo::new_from_path(repo_path).unwrap();
        let lines = git_info.get_lines().unwrap();

        // Find the staged file section and collapse it
        let mut collapsed_sections = HashSet::new();
        for line in &lines {
            if let Some(section) = &line.section {
                if let SectionType::StagedFile { path } = section {
                    if path == "test.txt" {
                        collapsed_sections.insert(section.clone());
                    }
                }
            }
        }

        // Verify we found and collapsed the file
        assert!(
            collapsed_sections.contains(&SectionType::StagedFile {
                path: "test.txt".to_string()
            }),
            "Should have found and collapsed the staged file"
        );

        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines,
                cursor_position: 0,
                scroll_offset: 0,
                viewport_height: 20,
                collapsed_sections,
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Unstage all files
        let follow_up = update(&mut model, Message::UnstageAll);
        assert_eq!(follow_up, Some(Message::Refresh));

        // Process the refresh
        update(&mut model, Message::Refresh);

        // Verify the file is now in unstaged section and still collapsed
        assert!(
            model
                .ui_model
                .collapsed_sections
                .contains(&SectionType::UnstagedFile {
                    path: "test.txt".to_string()
                }),
            "Unstaged file should be collapsed after UnstageAll"
        );

        // The old staged file section should be cleaned up
        assert!(
            !model
                .ui_model
                .collapsed_sections
                .contains(&SectionType::StagedFile {
                    path: "test.txt".to_string()
                }),
            "Old staged file section should be cleaned up"
        );
    }

    #[test]
    fn test_expanded_state_preserved_when_staging() {
        use crate::git::test_repo::TestRepo;
        use std::fs;

        // Create a test repo with a modified file
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();

        // Modify the tracked file
        let file_path = repo_path.join("test.txt");
        fs::write(&file_path, "modified content").unwrap();

        // Create GitInfo from test repo
        let git_info = GitInfo::new_from_path(repo_path).unwrap();
        let lines = git_info.get_lines().unwrap();

        // Don't collapse the file - leave it expanded
        let collapsed_sections = HashSet::new();

        let mut model = Model {
            git_info,
            running_state: RunningState::Running,
            ui_model: UiModel {
                lines,
                cursor_position: 0,
                scroll_offset: 0,
                viewport_height: 20,
                collapsed_sections,
            },
            theme: Theme::default(),
            dialog: None,
            toast: None,
        };

        // Stage all modified files
        let follow_up = update(&mut model, Message::StageAllModified);
        assert_eq!(follow_up, Some(Message::Refresh));

        // Process the refresh
        update(&mut model, Message::Refresh);

        // Verify the file is NOT collapsed (stayed expanded)
        assert!(
            !model
                .ui_model
                .collapsed_sections
                .contains(&SectionType::StagedFile {
                    path: "test.txt".to_string()
                }),
            "File should remain expanded when moving"
        );
    }
}
