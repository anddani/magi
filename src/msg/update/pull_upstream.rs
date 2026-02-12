use crate::{
    model::{
        Model,
        arguments::Arguments::PullArguments,
        popup::{PopupContent, PopupContentCommand},
    },
    msg::Message,
};

use super::pty_helper::execute_pty_command;

pub fn update(model: &mut Model) -> Option<Message> {
    // Get the upstream from popup state
    let upstream =
        if let Some(PopupContent::Command(PopupContentCommand::Pull(ref state))) = model.popup {
            state.upstream.clone()
        } else {
            return None;
        }?;

    // Parse upstream into remote and branch (e.g., "origin/main" -> ("origin", "main"))
    let (remote, branch) = if let Some((r, b)) = upstream.split_once('/') {
        (r.to_string(), b.to_string())
    } else {
        // If no slash, assume it's just the remote name
        (upstream.clone(), String::new())
    };

    let mut args = vec!["pull".to_string(), "-v".to_string(), remote];

    // Add branch if specified
    if !branch.is_empty() {
        args.push(branch);
    }

    if let Some(PullArguments(arguments)) = model.arguments.take() {
        args.extend(arguments.into_iter().map(|a| a.flag().to_string()));
    }

    execute_pty_command(model, args, format!("Pull from {}", upstream))
}
