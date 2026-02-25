use crate::{
    git::{
        config::set_push_remote,
        push::{get_current_branch, parse_remote_branch, set_upstream_branch},
    },
    model::{
        Model,
        arguments::Arguments::PullArguments,
        popup::{PopupContent, PopupContentCommand},
    },
    msg::{Message, PullCommand},
};

use super::pty_helper::execute_pty_command;

pub fn update(model: &mut Model, pull_command: PullCommand) -> Option<Message> {
    let extra_args = if let Some(PullArguments(arguments)) = model.arguments.take() {
        arguments
            .into_iter()
            .map(|a| a.flag().to_string())
            .collect()
    } else {
        vec![]
    };
    match pull_command {
        PullCommand::PullFromPushRemote(remote) => pull_from_push_remote(model, remote, extra_args),
        // TODO: Maybe merge this with the other somehow?
        PullCommand::PullUpstream => pull_from_upstream(model, extra_args),
        PullCommand::PullFromUpstream(upstream) => {
            pull_from_up_stream_setting(model, upstream, extra_args)
        }
    }
}

/// Pull from the given remote, treating it as the push remote.
/// Sets `branch.<name>.pushRemote` to the remote, then runs `git pull -v <remote> <current_branch>`.
fn pull_from_push_remote(
    model: &mut Model,
    remote: String,
    extra_args: Vec<String>,
) -> Option<Message> {
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

    args.extend(extra_args);

    let operation_name = format!("Pull from {}/{}", remote, current_branch);

    execute_pty_command(model, args, operation_name)
}

fn pull_from_upstream(model: &mut Model, extra_args: Vec<String>) -> Option<Message> {
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

    args.extend(extra_args);

    execute_pty_command(model, args, format!("Pull from {}", upstream))
}

fn pull_from_up_stream_setting(
    model: &mut Model,
    upstream: String,
    extra_args: Vec<String>,
) -> Option<Message> {
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

    args.extend(extra_args);

    let operation_name = format!("Pull from {}", upstream);

    execute_pty_command(model, args, operation_name)
}
