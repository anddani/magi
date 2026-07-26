use std::path::PathBuf;

use crate::{
    git::worktree::{WorktreeResult, main_worktree_path, worktree_delete},
    model::{Model, popup::PopupContent},
    msg::Message,
    msg::update::worktree_checkout::switch_to_worktree,
};

pub fn update(model: &mut Model, worktree: String) -> Option<Message> {
    // Resolve the main working tree before deleting: the worktree being
    // deleted may be the one we are currently in, in which case its path
    // (and thus `git -C <workdir>`) is gone afterwards.
    let main_path = main_worktree_path(&model.workdir)
        .map(PathBuf::from)
        .unwrap_or_else(|| model.workdir.clone());

    match worktree_delete(&main_path, &worktree) {
        Ok(WorktreeResult::Success) => {
            model.popup = None;
            // If the deleted worktree is the one we are currently in, fall
            // back to the main working tree.
            if !model.workdir.exists() {
                switch_to_worktree(model, main_path);
            }
            Some(Message::Refresh)
        }
        Ok(WorktreeResult::Error(err)) => {
            model.popup = Some(PopupContent::Error { message: err });
            None
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Worktree delete failed: {err}"),
            });
            None
        }
    }
}
