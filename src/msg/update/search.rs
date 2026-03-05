use crate::{
    model::{Line, LineContent, Model},
    msg::{Message, util::visible_lines_between},
};

/// Extract searchable plain text from a line's content.
fn line_searchable_text(line: &Line) -> String {
    match &line.content {
        LineContent::EmptyLine => String::new(),
        LineContent::HeadRef(r) | LineContent::MergeRef(r) | LineContent::PushRef(r) => {
            format!("{} {} {}", r.name, r.commit_hash, r.commit_summary)
        }
        LineContent::Tag(t) => format!("{} ({})", t.name, t.commits_ahead),
        LineContent::SectionHeader { title, count } => match count {
            Some(c) => format!("{} ({})", title, c),
            None => title.clone(),
        },
        LineContent::UnpulledSectionHeader { remote_name, count } => {
            format!("Unpulled from {} ({})", remote_name, count)
        }
        LineContent::UntrackedFile(path) => path.clone(),
        LineContent::UnstagedFile(fc) | LineContent::StagedFile(fc) => fc.path.clone(),
        LineContent::DiffHunk(h) => h.header.clone(),
        LineContent::DiffLine(dl) => dl.content.clone(),
        LineContent::Commit(ci) => {
            let refs: Vec<&str> = ci.refs.iter().map(|r| r.name.as_str()).collect();
            format!("{} {} {}", ci.hash, refs.join(" "), ci.message)
        }
        LineContent::LogLine(le) => {
            let refs: Vec<&str> = le.refs.iter().map(|r| r.name.as_str()).collect();
            format!(
                "{} {} {} {}",
                le.graph,
                le.hash.as_deref().unwrap_or(""),
                refs.join(" "),
                le.message.as_deref().unwrap_or("")
            )
        }
        LineContent::Stash(se) => se.message.clone(),
        LineContent::RevertingEntry { hash, message, .. } => format!("{} {}", hash, message),
        LineContent::RebasingEntry { hash, message, .. } => format!("{} {}", hash, message),
        LineContent::PreviewLine { content, .. } => content.clone(),
    }
}

/// Returns true if the line contains the query string (case-sensitive, consecutive).
fn line_matches(line: &Line, query: &str) -> bool {
    if query.is_empty() {
        return false;
    }
    line_searchable_text(line).contains(query)
}

/// Adjusts scroll_offset so the cursor is visible within the viewport.
fn ensure_cursor_visible(model: &mut Model) {
    let cursor = model.ui_model.cursor_position;
    let viewport_height = model.ui_model.viewport_height;

    if viewport_height == 0 {
        return;
    }

    // Scroll up if cursor is above viewport
    if cursor < model.ui_model.scroll_offset {
        model.ui_model.scroll_offset = cursor;
        return;
    }

    // Scroll down if cursor is below viewport
    let visible_before_cursor = visible_lines_between(
        &model.ui_model.lines,
        model.ui_model.scroll_offset,
        cursor,
        &model.ui_model.collapsed_sections,
    );

    if visible_before_cursor >= viewport_height {
        let mut new_scroll = cursor;
        let mut count = 0usize;
        while new_scroll > 0 && count < viewport_height - 1 {
            new_scroll -= 1;
            if !model.ui_model.lines[new_scroll].is_hidden(&model.ui_model.collapsed_sections) {
                count += 1;
            }
        }
        model.ui_model.scroll_offset = new_scroll;
    }
}

pub fn input_char(model: &mut Model, c: char) -> Option<Message> {
    model.ui_model.search_query.push(c);
    None
}

pub fn input_backspace(model: &mut Model) -> Option<Message> {
    model.ui_model.search_query.pop();
    None
}

pub fn confirm(model: &mut Model) -> Option<Message> {
    model.ui_model.search_mode_active = false;
    if model.ui_model.search_query.is_empty() {
        return None;
    }
    // Jump to first match at or after current cursor position
    jump_to_match(model, true, true);
    None
}

pub fn next(model: &mut Model) -> Option<Message> {
    if model.ui_model.search_query.is_empty() {
        return None;
    }
    jump_to_match(model, true, false);
    None
}

pub fn prev(model: &mut Model) -> Option<Message> {
    if model.ui_model.search_query.is_empty() {
        return None;
    }
    jump_to_match(model, false, false);
    None
}

pub fn cancel(model: &mut Model) -> Option<Message> {
    model.ui_model.search_query.clear();
    model.ui_model.search_mode_active = false;
    None
}

/// Jump to the next or previous visible match from the current cursor position.
///
/// - `forward`: if true, search forward (wrap around); if false, search backward.
/// - `include_current`: if true, also consider the current cursor line as a candidate.
fn jump_to_match(model: &mut Model, forward: bool, include_current: bool) {
    let query = model.ui_model.search_query.clone();
    let total = model.ui_model.lines.len();
    let cursor = model.ui_model.cursor_position;

    if forward {
        // Search from cursor+1 (or cursor if include_current) wrapping around
        let start = if include_current { cursor } else { cursor + 1 };
        for i in 0..total {
            let idx = (start + i) % total;
            let line = &model.ui_model.lines[idx];
            if line.is_hidden(&model.ui_model.collapsed_sections) {
                continue;
            }
            if line_matches(line, &query) {
                model.ui_model.cursor_position = idx;
                ensure_cursor_visible(model);
                return;
            }
        }
    } else {
        // Search backward from cursor-1 wrapping around
        for i in 1..=total {
            let idx = (cursor + total - i) % total;
            let line = &model.ui_model.lines[idx];
            if line.is_hidden(&model.ui_model.collapsed_sections) {
                continue;
            }
            if line_matches(line, &query) {
                model.ui_model.cursor_position = idx;
                ensure_cursor_visible(model);
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Theme;
    use crate::git::GitInfo;
    use crate::git::test_repo::TestRepo;
    use crate::model::ViewMode;
    use crate::model::{RunningState, UiModel};

    fn make_line(text: &str) -> Line {
        Line {
            content: LineContent::UntrackedFile(text.to_string()),
            section: None,
        }
    }

    fn create_test_model_with_lines(lines: Vec<Line>) -> Model {
        let test_repo = TestRepo::new();
        let repo_path = test_repo.repo.workdir().unwrap();
        let git_info = GitInfo::new_from_path(repo_path).unwrap();
        let workdir = repo_path.to_path_buf();
        let mut ui = UiModel::default();
        ui.lines = lines;
        ui.viewport_height = 10;
        Model {
            git_info,
            workdir,
            running_state: RunningState::Running,
            ui_model: ui,
            theme: Theme::default(),
            popup: None,
            toast: None,
            select_result: None,
            select_context: None,
            pty_state: None,
            arg_mode: false,
            pending_g: false,
            arguments: None,
            open_pr_branch: None,
            view_mode: ViewMode::Status,
            cursor_reposition_context: None,
            preview_return_mode: None,
            preview_return_cursor: 0,
        }
    }

    #[test]
    fn test_input_char_appends_to_query() {
        let mut model = create_test_model_with_lines(vec![]);
        input_char(&mut model, 'f');
        input_char(&mut model, 'o');
        input_char(&mut model, 'o');
        assert_eq!(model.ui_model.search_query, "foo");
    }

    #[test]
    fn test_input_backspace_removes_last_char() {
        let mut model = create_test_model_with_lines(vec![]);
        model.ui_model.search_query = "foo".to_string();
        input_backspace(&mut model);
        assert_eq!(model.ui_model.search_query, "fo");
    }

    #[test]
    fn test_cancel_clears_query_and_mode() {
        let mut model = create_test_model_with_lines(vec![]);
        model.ui_model.search_query = "foo".to_string();
        model.ui_model.search_mode_active = true;
        cancel(&mut model);
        assert_eq!(model.ui_model.search_query, "");
        assert!(!model.ui_model.search_mode_active);
    }

    #[test]
    fn test_confirm_disables_search_mode() {
        let mut model = create_test_model_with_lines(vec![make_line("hello")]);
        model.ui_model.search_query = "hello".to_string();
        model.ui_model.search_mode_active = true;
        confirm(&mut model);
        assert!(!model.ui_model.search_mode_active);
    }

    #[test]
    fn test_next_jumps_to_match() {
        let lines = vec![make_line("apple"), make_line("banana"), make_line("cherry")];
        let mut model = create_test_model_with_lines(lines);
        model.ui_model.search_query = "banana".to_string();
        model.ui_model.cursor_position = 0;
        next(&mut model);
        assert_eq!(model.ui_model.cursor_position, 1);
    }

    #[test]
    fn test_next_wraps_around() {
        let lines = vec![make_line("apple"), make_line("banana"), make_line("cherry")];
        let mut model = create_test_model_with_lines(lines);
        model.ui_model.search_query = "apple".to_string();
        model.ui_model.cursor_position = 2;
        next(&mut model);
        assert_eq!(model.ui_model.cursor_position, 0);
    }

    #[test]
    fn test_prev_jumps_to_previous_match() {
        let lines = vec![make_line("apple"), make_line("banana"), make_line("cherry")];
        let mut model = create_test_model_with_lines(lines);
        model.ui_model.search_query = "apple".to_string();
        model.ui_model.cursor_position = 2;
        prev(&mut model);
        assert_eq!(model.ui_model.cursor_position, 0);
    }

    #[test]
    fn test_prev_wraps_around() {
        let lines = vec![make_line("apple"), make_line("banana"), make_line("cherry")];
        let mut model = create_test_model_with_lines(lines);
        model.ui_model.search_query = "cherry".to_string();
        model.ui_model.cursor_position = 0;
        prev(&mut model);
        assert_eq!(model.ui_model.cursor_position, 2);
    }

    #[test]
    fn test_next_no_match_leaves_cursor_unchanged() {
        let lines = vec![make_line("apple"), make_line("banana")];
        let mut model = create_test_model_with_lines(lines);
        model.ui_model.search_query = "xyz".to_string();
        model.ui_model.cursor_position = 0;
        next(&mut model);
        assert_eq!(model.ui_model.cursor_position, 0);
    }

    #[test]
    fn test_confirm_jumps_to_first_match() {
        let lines = vec![make_line("apple"), make_line("banana"), make_line("cherry")];
        let mut model = create_test_model_with_lines(lines);
        model.ui_model.search_query = "cherry".to_string();
        model.ui_model.search_mode_active = true;
        model.ui_model.cursor_position = 0;
        confirm(&mut model);
        assert_eq!(model.ui_model.cursor_position, 2);
    }
}
