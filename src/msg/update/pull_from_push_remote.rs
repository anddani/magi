use crate::{
    git::{config::set_push_remote, push::get_current_branch},
    model::{Model, arguments::Arguments::PullArguments, popup::PopupContent},
    msg::Message,
};

use super::pty_helper::execute_pty_command;

/// Pull from the given remote, treating it as the push remote.
/// Sets `branch.<name>.pushRemote` to the remote, then runs `git pull -v <remote> <current_branch>`.
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

    let mut args = vec![
        "pull".to_string(),
        "-v".to_string(),
        remote.clone(),
        current_branch.clone(),
    ];

    if let Some(PullArguments(arguments)) = model.arguments.take() {
        args.extend(arguments.into_iter().map(|a| a.flag().to_string()));
    }

    let operation_name = format!("Pull from {}/{}", remote, current_branch);

    execute_pty_command(model, args, operation_name)
}
