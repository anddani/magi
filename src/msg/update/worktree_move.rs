use crate::{
    git::worktree::{WorktreeResult, worktree_move},
    model::{Model, popup::PopupContent},
    msg::Message,
    msg::update::worktree_checkout::{resolve_path, switch_to_worktree},
};

pub fn update(model: &mut Model, worktree: String, path: String) -> Option<Message> {
    match worktree_move(&model.workdir, &worktree, &path) {
        Ok(WorktreeResult::Success) => {
            // If the moved worktree is the one we are currently in, its old
            // path no longer exists — follow it to the new location.
            if !model.workdir.exists() {
                let new_path = resolve_path(&model.workdir, &path);
                switch_to_worktree(model, new_path);
            }
            Some(Message::Refresh)
        }
        Ok(WorktreeResult::Error(err)) => {
            model.popup = Some(PopupContent::Error { message: err });
            None
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Worktree move failed: {err}"),
            });
            None
        }
    }
}
