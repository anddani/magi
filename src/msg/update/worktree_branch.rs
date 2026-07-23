use crate::{
    git::worktree::{WorktreeAddResult, worktree_add_branch},
    model::{Model, popup::PopupContent},
    msg::Message,
    msg::update::worktree_checkout::{resolve_path, switch_to_worktree},
};

pub fn update(
    model: &mut Model,
    starting_point: String,
    branch_name: String,
    path: String,
) -> Option<Message> {
    match worktree_add_branch(&model.workdir, &path, &branch_name, &starting_point) {
        Ok(WorktreeAddResult::Success) => {
            let worktree_path = resolve_path(&model.workdir, &path);
            switch_to_worktree(model, worktree_path);
            Some(Message::Refresh)
        }
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
