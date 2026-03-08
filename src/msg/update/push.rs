use crate::{
    git::{
        config::set_push_remote,
        push::{get_current_branch, get_upstream_branch, parse_remote_branch},
    },
    model::{Model, arguments::Arguments::PushArguments, popup::PopupContent},
    msg::{Message, PushCommand, update::pty_helper::execute_pty_command},
};

pub fn update(model: &mut Model, push_command: PushCommand) -> Option<Message> {
    let extra_args = if let Some(PushArguments(arguments)) = model.arguments.take() {
        arguments
            .into_iter()
            .map(|a| a.flag().to_string())
            .collect()
    } else {
        vec![]
    };
    match push_command {
        // TODO: Merge with below
        PushCommand::PushUpstream => push_to_upstream(model, extra_args),
        PushCommand::PushToRemote(upstream) => {
            push_to_upstream_setting(model, upstream, extra_args)
        }
        PushCommand::PushToPushRemote(remote) => push_to_push_remote(model, remote, extra_args),
        PushCommand::PushAllTags(remote) => push_all_tags(model, remote, extra_args),
        PushCommand::PushTag(tag) => push_tag(model, tag, extra_args),
        PushCommand::PushElsewhere(upstream) => push_to_elsewhere(model, upstream, extra_args),
        PushCommand::PushOtherBranch { local, remote } => {
            push_other_branch(model, local, remote, extra_args)
        }
        PushCommand::PushRefspecs { remote, refspecs } => {
            push_refspecs(model, remote, refspecs, extra_args)
        }
        PushCommand::PushMatching(remote) => push_matching(model, remote, extra_args),
    }
}

fn push_to_upstream(model: &mut Model, extra_args: Vec<String>) -> Option<Message> {
    // When the upstream branch name differs from the local branch name, git push
    // refuses to run without an explicit refspec. Detect this and supply one.
    if let (Ok(Some(upstream)), Ok(Some(local))) = (
        get_upstream_branch(&model.workdir),
        get_current_branch(&model.workdir),
    ) {
        let (remote, remote_branch) = parse_remote_branch(&upstream);
        if !remote_branch.is_empty() && remote_branch != local {
            let refspec = format!("HEAD:{}", remote_branch);
            let mut args = vec![remote, refspec];
            args.extend(extra_args);
            return execute_push(model, args, "Push".to_string());
        }
    }

    execute_push(model, extra_args, "Push".to_string())
}

fn push_to_upstream_setting(
    model: &mut Model,
    upstream: String,
    extra_args: Vec<String>,
) -> Option<Message> {
    let (remote, branch) = parse_remote_branch(&upstream);

    // Build the refspec for setting upstream
    let refspec = format!("HEAD:{}", branch);

    // Build extra arguments for setting upstream
    let mut args = vec!["--set-upstream".to_string(), remote, refspec];

    args.extend(extra_args);

    let operation_name = format!("Push to {}", upstream);

    execute_push(model, args, operation_name)
}

/// Push to the given remote, treating it as the push remote.
/// Sets `branch.<name>.pushRemote` to the remote, then runs `git push -v <remote> <current_branch>`.
fn push_to_push_remote(
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

    let operation_name = format!("Push to {}/{}", remote, current_branch);

    let mut args = vec![remote, current_branch];
    args.extend(extra_args);

    execute_push(model, args, operation_name)
}

fn push_all_tags(model: &mut Model, remote: String, extra_args: Vec<String>) -> Option<Message> {
    let mut args = vec![remote, "--tags".to_string()];
    args.extend(extra_args);
    execute_push(model, args, "Push tags".to_string())
}

fn push_to_elsewhere(
    model: &mut Model,
    upstream: String,
    extra_args: Vec<String>,
) -> Option<Message> {
    let (remote, branch) = parse_remote_branch(&upstream);
    let mut args = if branch.is_empty() {
        vec![remote]
    } else {
        vec![remote, format!("HEAD:{}", branch)]
    };
    args.extend(extra_args);
    execute_push(model, args, format!("Push to {}", upstream))
}

fn push_refspecs(
    model: &mut Model,
    remote: String,
    refspecs: String,
    extra_args: Vec<String>,
) -> Option<Message> {
    let mut args: Vec<String> = vec![remote.clone()];
    for spec in refspecs.split(',') {
        let spec = spec.trim();
        if !spec.is_empty() {
            args.push(spec.to_string());
        }
    }
    args.extend(extra_args);
    execute_push(model, args, format!("Push refspecs to {}", remote))
}

fn push_other_branch(
    model: &mut Model,
    local: String,
    remote: String,
    extra_args: Vec<String>,
) -> Option<Message> {
    let (remote_name, branch) = parse_remote_branch(&remote);
    let mut args = if branch.is_empty() {
        vec![remote_name, local.clone()]
    } else {
        vec![remote_name, format!("{}:{}", local, branch)]
    };
    args.extend(extra_args);
    execute_push(model, args, format!("Push {} to {}", local, remote))
}

fn push_matching(model: &mut Model, remote: String, extra_args: Vec<String>) -> Option<Message> {
    let mut args = vec![remote.clone(), ":".to_string()];
    args.extend(extra_args);
    execute_push(model, args, format!("Push matching branches to {}", remote))
}

fn push_tag(model: &mut Model, tag: String, extra_args: Vec<String>) -> Option<Message> {
    let mut args = vec!["origin".to_string(), tag.clone()];
    args.extend(extra_args);

    execute_push(model, args, format!("Push tag {}", tag))
}

/// Execute a push command with the given extra arguments.
///
/// This handles the common push logic:
/// - Building args with push arguments from model
/// - Calling the generic PTY command executor
///
/// # Arguments
/// * `model` - The application model
/// * `extra_args` - Additional arguments to pass to git push (e.g., `["--set-upstream", "origin", "HEAD:main"]`)
/// * `operation_name` - Name to display for this operation (e.g., "Push" or "Push to origin/main")
pub fn execute_push(
    model: &mut Model,
    extra_args: Vec<String>,
    operation_name: String,
) -> Option<Message> {
    // Build push command arguments
    let mut args = vec!["push".to_string(), "-v".to_string()];

    // Add push arguments from model (e.g., --force-with-lease, --force)
    if let Some(PushArguments(arguments)) = model.arguments.take() {
        args.extend(arguments.into_iter().map(|a| a.flag().to_string()));
    }

    // Add extra arguments
    args.extend(extra_args);

    execute_pty_command(model, args, operation_name)
}
