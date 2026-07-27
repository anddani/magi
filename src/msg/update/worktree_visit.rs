use std::path::PathBuf;

use crate::{
    model::{Model, popup::PopupContent},
    msg::Message,
    msg::update::worktree_checkout::switch_to_worktree,
};

pub fn update(model: &mut Model, worktree: String) -> Option<Message> {
    let path = PathBuf::from(&worktree);
    if !path.exists() {
        model.popup = Some(PopupContent::Error {
            message: format!("Worktree \"{worktree}\" does not exist"),
        });
        return None;
    }
    switch_to_worktree(model, path);
    Some(Message::Refresh)
}
