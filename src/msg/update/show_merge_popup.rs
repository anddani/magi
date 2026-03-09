use std::path::Path;

use crate::{
    model::{
        Model,
        popup::{MergePopupState, PopupContent, PopupContentCommand},
    },
    msg::Message,
};

/// Returns true if a merge sequence is currently in progress.
/// Checks for `.git/MERGE_HEAD`.
pub fn merge_in_progress(workdir: &Path) -> bool {
    workdir.join(".git").join("MERGE_HEAD").exists()
}

pub fn update(model: &mut Model) -> Option<Message> {
    let in_progress = merge_in_progress(&model.workdir);
    let state = MergePopupState { in_progress };
    model.popup = Some(PopupContent::Command(PopupContentCommand::Merge(state)));
    None
}
