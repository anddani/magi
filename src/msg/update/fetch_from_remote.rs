use crate::{
    git::push::parse_remote_branch,
    model::{Model, arguments::Arguments::FetchArguments},
    msg::Message,
};

use super::pty_helper::execute_pty_command;

pub fn update(model: &mut Model, upstream: String) -> Option<Message> {
    let (remote, branch) = parse_remote_branch(&upstream);

    let mut args = vec!["fetch".to_string(), "-v".to_string(), remote];

    // Add branch if specified
    if !branch.is_empty() {
        args.push(branch);
    }

    if let Some(FetchArguments(arguments)) = model.arguments.take() {
        args.extend(arguments.into_iter().map(|a| a.flag().to_string()));
    }

    let operation_name = format!("Fetch from {}", upstream);

    execute_pty_command(model, args, operation_name)
}
