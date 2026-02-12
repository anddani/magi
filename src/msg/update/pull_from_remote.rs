use crate::{
    git::push::{parse_remote_branch, set_upstream_branch},
    model::{Model, arguments::Arguments::PullArguments, popup::PopupContent},
    msg::Message,
};

use super::pty_helper::execute_pty_command;

pub fn update(model: &mut Model, upstream: String) -> Option<Message> {
    let (remote, branch) = parse_remote_branch(&upstream);

    // Set the upstream branch configuration first
    if let Err(e) = set_upstream_branch(&model.git_info.repository, &upstream) {
        model.popup = Some(PopupContent::Error {
            message: format!("Failed to set upstream: {}", e),
        });
        return None;
    }

    let mut args = vec!["pull".to_string(), "-v".to_string(), remote];

    // Add branch if specified
    if !branch.is_empty() {
        args.push(branch);
    }

    if let Some(PullArguments(arguments)) = model.arguments.take() {
        args.extend(arguments.into_iter().map(|a| a.flag().to_string()));
    }

    let operation_name = format!("Pull from {}", upstream);

    execute_pty_command(model, args, operation_name)
}
