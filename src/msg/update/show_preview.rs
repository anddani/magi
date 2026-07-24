use crate::git::preview::{get_commit_preview_lines, get_stash_preview_lines};
use crate::model::{Line, LineContent, Model, ViewMode};
use crate::msg::Message;

pub fn update(model: &mut Model) -> Option<Message> {
    let cursor_line = model.ui_model.lines.get(model.ui_model.cursor_position)?;

    let preview_lines = match &cursor_line.content {
        LineContent::Commit(info) => get_commit_preview_lines(&model.workdir, &info.hash),
        LineContent::LogLine(entry) => {
            let hash = entry.hash.as_deref()?;
            get_commit_preview_lines(&model.workdir, hash)
        }
        LineContent::Stash(stash) => get_stash_preview_lines(&model.workdir, stash.index),
        LineContent::RebaseTodoLine(entry) => get_commit_preview_lines(&model.workdir, &entry.hash),
        _ => return None,
    };

    enter_preview(model, preview_lines)
}

/// Shows the diff of a specific stash (by index) in the preview view.
pub fn show_stash(model: &mut Model, index: usize) -> Option<Message> {
    model.popup = None;
    let preview_lines = get_stash_preview_lines(&model.workdir, index);
    enter_preview(model, preview_lines)
}

fn enter_preview(model: &mut Model, preview_lines: Vec<Line>) -> Option<Message> {
    if preview_lines.is_empty() {
        return None;
    }

    model.preview_return_mode = Some(model.view_mode.clone());
    model.preview_return_ui_model = Some(model.ui_model.clone());
    model.ui_model.lines = preview_lines;
    model.ui_model.cursor_position = 0;
    model.ui_model.scroll_offset = 0;
    model.view_mode = ViewMode::Preview;
    None
}
