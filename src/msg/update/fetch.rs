use crate::{
    git::{
        config::set_push_remote,
        push::{get_current_branch, parse_remote_branch},
    },
    model::{
        Model,
        arguments::{Arguments::FetchArguments, PopupArgument},
        popup::{PopupContent, PopupContentCommand},
    },
    msg::{FetchCommand, Message},
};

use super::pty_helper::execute_pty_command;

pub fn update(model: &mut Model, fetch_command: FetchCommand) -> Option<Message> {
    let extra_args = if let Some(FetchArguments(arguments)) = model.arguments.take() {
        arguments
            .into_iter()
            .map(|a| a.flag().to_string())
            .collect()
    } else {
        vec![]
    };
    match fetch_command {
        FetchCommand::FetchUpstream => fetch_upstream(model, extra_args),
        FetchCommand::FetchFromRemoteBranch(upstream) => {
            fetch_from_remote_branch(model, upstream, extra_args)
        }
        FetchCommand::FetchFromPushRemote(remote) => {
            fetch_from_push_remote(model, remote, extra_args)
        }
        FetchCommand::FetchAllRemotes => fetch_from_all_remotes(model, extra_args),
        FetchCommand::FetchModules => fetch_submodules(model, extra_args),
        FetchCommand::FetchRefspecs { remote, refspecs } => {
            fetch_refspecs(model, remote, refspecs, extra_args)
        }
    }
}

fn fetch_upstream(model: &mut Model, extra_args: Vec<String>) -> Option<Message> {
    // Get the upstream from popup state
    let upstream =
        if let Some(PopupContent::Command(PopupContentCommand::Fetch(ref state))) = model.popup {
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

    let mut args = vec!["fetch".to_string(), "-v".to_string(), remote];

    // Add branch if specified
    if !branch.is_empty() {
        args.push(branch);
    }

    args.extend(extra_args);

    execute_pty_command(model, args, format!("Fetch from {}", upstream))
}

fn fetch_from_remote_branch(
    model: &mut Model,
    remote_branch: String,
    extra_args: Vec<String>,
) -> Option<Message> {
    let (remote, branch) = parse_remote_branch(&remote_branch);

    let mut args = vec!["fetch".to_string(), "-v".to_string(), remote];

    // Add branch if specified
    if !branch.is_empty() {
        args.push(branch);
    }

    args.extend(extra_args);

    let operation_name = format!("Fetch from {}", remote_branch);

    execute_pty_command(model, args, operation_name)
}

/// Fetch from the given remote, treating it as the push remote.
/// Sets `branch.<name>.pushRemote` to the remote, then runs `git fetch -v <remote>`.
fn fetch_from_push_remote(
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

    // Set branch.<name>.pushRemote in git config
    if let Err(e) = set_push_remote(&model.git_info.repository, &current_branch, &remote) {
        model.popup = Some(PopupContent::Error {
            message: format!("Failed to set push remote: {}", e),
        });
        return None;
    }

    let mut args = vec!["fetch".to_string(), "-v".to_string(), remote.clone()];

    args.extend(extra_args);

    let operation_name = format!("Fetch from {}", remote);

    execute_pty_command(model, args, operation_name)
}

fn fetch_from_all_remotes(model: &mut Model, extra_args: Vec<String>) -> Option<Message> {
    let mut args = vec!["fetch".to_string(), "-v".to_string(), "--all".to_string()];

    args.extend(extra_args);

    execute_pty_command(model, args, "Fetch all".to_string())
}

fn fetch_refspecs(
    model: &mut Model,
    remote: String,
    refspecs: String,
    extra_args: Vec<String>,
) -> Option<Message> {
    let mut args: Vec<String> = vec!["fetch".to_string(), "-v".to_string(), remote.clone()];
    for spec in refspecs.split(',') {
        let spec = spec.trim();
        if !spec.is_empty() {
            args.push(spec.to_string());
        }
    }
    args.extend(extra_args);
    execute_pty_command(model, args, format!("Fetch refspecs from {}", remote))
}

fn fetch_submodules(model: &mut Model, extra_args: Vec<String>) -> Option<Message> {
    let mut args = vec![
        "fetch".to_string(),
        "-v".to_string(),
        "--recurse-submodules".to_string(),
    ];

    args.extend(extra_args);

    execute_pty_command(model, args, "Fetch submodules".to_string())
}
