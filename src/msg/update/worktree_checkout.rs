use std::path::PathBuf;

use crate::{
    git::{GitInfo, worktree::{WorktreeAddResult, worktree_add}},
    model::{Model, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model, branch: String, path: String, checkout: bool) -> Option<Message> {
    match worktree_add(&model.workdir, &path, &branch) {
        Ok(WorktreeAddResult::Success) => {
            if checkout {
                let worktree_path = resolve_path(&model.workdir, &path);
                switch_to_worktree(model, worktree_path);
            }
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

fn resolve_path(workdir: &std::path::Path, path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        p
    } else {
        workdir.join(p)
    }
}

fn switch_to_worktree(model: &mut Model, path: PathBuf) {
    match GitInfo::new_from_path(&path) {
        Ok(git_info) => {
            // Extract canonical workdir from the new repo
            if let Some(workdir) = git_info.repository.workdir() {
                model.workdir = workdir.to_path_buf();
            } else {
                model.workdir = path;
            }
            model.git_info = git_info;
        }
        Err(_) => {
            // If we can't open the new repo, fall back to just refreshing current view
        }
    }
}
