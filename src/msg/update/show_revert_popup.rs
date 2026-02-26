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
    };
    model.popup = Some(PopupContent::Command(PopupContentCommand::Revert(state)));
    None
}

/// Collect commit hashes from the visual selection or cursor position.
/// Returns an empty vec if the selection contains non-commit lines.
fn collect_selected_commits(model: &Model) -> Vec<String> {
    if let Some((start, end)) = model.ui_model.visual_selection_range() {
        // Visual mode: every line in the range must be a commit
        let range = &model.ui_model.lines[start..=end];
        let commits: Vec<String> = range
            .iter()
            .filter_map(|line| {
                if let LineContent::Commit(info) = &line.content {
                    Some(info.hash.clone())
                } else {
                    None
                }
            })
            .collect();

        // Only accept if ALL lines in the selection are commits
        if commits.len() == range.len() {
            commits
        } else {
            vec![]
        }
    } else {
        // Normal mode: cursor line must be a commit
        let cursor = model.ui_model.cursor_position;
        if let Some(line) = model.ui_model.lines.get(cursor)
            && let LineContent::Commit(info) = &line.content
        {
            return vec![info.hash.clone()];
        }
        vec![]
    }
}
