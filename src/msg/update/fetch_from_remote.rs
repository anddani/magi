use crate::{
    model::{arguments::Arguments::FetchArguments, Model},
    msg::Message,
};

use super::pty_helper::execute_pty_command;

/// Parse remote/branch into components.
/// e.g., "origin/main" -> ("origin", "main")
fn parse_remote_branch(upstream: &str) -> (String, String) {
    if let Some((remote, branch)) = upstream.split_once('/') {
        (remote.to_string(), branch.to_string())
    } else {
        // If no slash, assume it's just the remote name
        (upstream.to_string(), String::new())
    }
}

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
