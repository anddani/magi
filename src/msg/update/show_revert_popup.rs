use crate::{
    git::revert::revert_in_progress,
    model::{
        LineContent, Model,
        popup::{PopupContent, PopupContentCommand, RevertPopupState},
    },
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let in_progress = revert_in_progress(&model.workdir);

    let selected_commits = if in_progress {
        vec![]
    } else {
        collect_selected_commits(model)
    };

    let state = RevertPopupState {
        in_progress,
        selected_commits,
        mainline: None,
    };
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(state)));
    None
}

/// Extract a commit hash from a line, if the line represents a commit.
/// Handles both status-view `Commit` lines and log-view `LogLine` entries.
fn hash_from_line(line: &crate::model::Line) -> Option<String> {
    match &line.content {
        LineContent::Commit(info) => Some(info.hash.clone()),
        LineContent::LogLine(entry) => entry.hash.clone(),
        _ => None,
    }
}

/// Collect commit hashes from the visual selection or cursor position.
/// Returns an empty vec if the selection contains non-commit lines.
fn collect_selected_commits(model: &Model) -> Vec<String> {
    if let Some((start, end)) = model.ui_model.visual_selection_range() {
        // Visual mode: every line in the range must be a commit
        let range = &model.ui_model.lines[start..=end];
        let commits: Vec<String> = range.iter().filter_map(hash_from_line).collect();

        // Only accept if ALL lines in the selection are commits
        if commits.len() == range.len() {
            commits
        } else {
            vec![]
        }
    } else {
        // Normal mode: cursor line must be a commit or log entry
        let cursor = model.ui_model.cursor_position;
        if let Some(hash) = model.ui_model.lines.get(cursor).and_then(hash_from_line) {
            return vec![hash];
        }
        vec![]
    }
}
