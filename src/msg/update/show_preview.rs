use crate::git::preview::{get_commit_preview_lines, get_stash_preview_lines};
use crate::model::{LineContent, Model, ViewMode};
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
        _ => return None,
    };

    if preview_lines.is_empty() {
        return None;
    }

    model.preview_return_mode = Some(model.view_mode);
    model.preview_return_cursor = model.ui_model.cursor_position;
    model.ui_model.lines = preview_lines;
    model.ui_model.cursor_position = 0;
    model.ui_model.scroll_offset = 0;
    model.view_mode = ViewMode::Preview;
    None
}
