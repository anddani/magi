use crate::{
    git::worktree::{WorktreeAddResult, worktree_add},
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model, branch: String, path: String) -> Option<Message> {
    match worktree_add(&model.workdir, &path, &branch) {
        Ok(WorktreeAddResult::Success) => Some(Message::Refresh),
        Ok(WorktreeAddResult::Error(err)) => {
            model.popup = Some(PopupContent::Error { message: err });
            None
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Worktree add failed: {err}"),
            });
            None
        }
    }
}
