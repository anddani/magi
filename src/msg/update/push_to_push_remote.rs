use crate::{
    git::push::{get_current_branch, set_push_remote},
    model::{Model, popup::PopupContent},
    msg::Message,
};

use super::push_helper::execute_push;

/// Push to the given remote, treating it as the push remote.
/// Sets `branch.<name>.pushRemote` to the remote, then runs `git push -v <remote> <current_branch>`.
pub fn update(model: &mut Model, remote: String) -> Option<Message> {
    let current_branch = match get_current_branch(&model.workdir).ok().flatten() {
        Some(branch) => branch,
        None => {
            model.popup = Some(PopupContent::Error {
                message: "No branch is checked out".to_string(),
            });
            return None;
        }
    };

    if let Err(e) = set_push_remote(&model.git_info.repository, &current_branch, &remote) {
        model.popup = Some(PopupContent::Error {
            message: format!("Failed to set push remote: {}", e),
        });
        return None;
    }

    let operation_name = format!("Push to {}/{}", remote, current_branch);

    execute_push(
        model,
        vec![remote, current_branch],
        operation_name,
    )
}
