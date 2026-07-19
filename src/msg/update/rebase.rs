use std::time::{Duration, Instant};

use crate::{
    git::{
        config::set_push_remote,
        push::{get_current_branch, set_upstream_branch},
        rebase::{self, CommitResult},
    },
    model::{
        Model, Toast, ToastStyle, ViewMode,
        popup::{PopupContent, PopupContentCommand},
    },
    msg::{Message, RebaseCommand, update::pty_helper::execute_pty_command},
};

pub fn update(model: &mut Model, rebase_command: RebaseCommand) -> Option<Message> {
    match rebase_command {
        RebaseCommand::OntoPushRemote(remote) => onto_push_remote(model, remote),
        RebaseCommand::OntoUpstream => onto_upstream(model),
        RebaseCommand::OntoUpstreamSetting(upstream) => onto_upstream_setting(model, upstream),
        RebaseCommand::Elsewhere(target) => elsewhere(model, target),
        RebaseCommand::Subset { newbase, start } => subset(model, newbase, start),
        RebaseCommand::ExecuteInteractive => execute_interactive(model),
        RebaseCommand::ModifyCommit(commit) => modify_commit(model, commit),
        RebaseCommand::RewordCommit(commit) => reword_commit(model, commit),
        RebaseCommand::Continue => continue_rebase(model),
        RebaseCommand::Skip => skip_rebase(model),
        RebaseCommand::Abort => abort_rebase(model),
    }
}

/// Runs the interactive rebase prepared in the todo editor. This is an
/// external command (the TUI is suspended) because git may open the user's
/// editor for reword/squash commit messages.
fn execute_interactive(model: &mut Model) -> Option<Message> {
    let Some(state) = model.rebase_todo.take() else {
        return Some(Message::Refresh);
    };
    model.view_mode = ViewMode::Status;

    match rebase::start_interactive_rebase(
        &model.workdir,
        &state.base,
        state.base_has_parent,
        &state.entries,
    ) {
        Ok(CommitResult { success, message }) => {
            model.toast = Some(Toast {
                message,
                style: if success {
                    ToastStyle::Success
                } else {
                    ToastStyle::Warning
                },
                expires_at: Instant::now() + TOAST_DURATION,
            });
        }
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: e.to_string(),
            });
        }
    }
    Some(Message::Refresh)
}

/// Starts an interactive rebase that stops at `commit` for editing. This is
/// an external command (the TUI is suspended) because git prints rebase
/// progress and stop instructions to the terminal.
fn modify_commit(model: &mut Model, commit: String) -> Option<Message> {
    model.popup = None;

    match rebase::run_modify_commit(&model.workdir, &commit) {
        Ok(CommitResult { success, message }) => {
            model.toast = Some(Toast {
                message,
                style: if success {
                    ToastStyle::Success
                } else {
                    ToastStyle::Warning
                },
                expires_at: Instant::now() + TOAST_DURATION,
            });
        }
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: e.to_string(),
            });
        }
    }
    Some(Message::Refresh)
}

/// Starts an interactive rebase that rewords `commit`. This is an external
/// command (the TUI is suspended) because git opens the user's editor for
/// the new commit message.
fn reword_commit(model: &mut Model, commit: String) -> Option<Message> {
    model.popup = None;

    match rebase::run_reword_commit(&model.workdir, &commit) {
        Ok(CommitResult { success, message }) => {
            model.toast = Some(Toast {
                message,
                style: if success {
                    ToastStyle::Success
                } else {
                    ToastStyle::Warning
                },
                expires_at: Instant::now() + TOAST_DURATION,
            });
        }
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: e.to_string(),
            });
        }
    }
    Some(Message::Refresh)
}

fn elsewhere(model: &mut Model, target: String) -> Option<Message> {
    let args = vec!["rebase".to_string(), target];
    execute_pty_command(model, args, "Rebase".to_string())
}

/// Rebase a subset of the current branch's history onto a new base:
/// `git rebase --onto <newbase> <start>^` (or `--root` when start has no parent).
fn subset(model: &mut Model, newbase: String, start: String) -> Option<Message> {
    let mut args = vec!["rebase".to_string(), "--onto".to_string(), newbase.clone()];
    if rebase::commit_has_parent(&model.workdir, &start) {
        args.push(format!("{}^", start));
    } else {
        args.push("--root".to_string());
    }
    execute_pty_command(model, args, format!("Rebase subset onto {}", newbase))
}

/// Rebase the current branch onto its push remote branch.
/// Sets `branch.<name>.pushRemote` to the remote, then runs
/// `git rebase <remote>/<current_branch>`.
fn onto_push_remote(model: &mut Model, remote: String) -> Option<Message> {
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

    let target = format!("{}/{}", remote, current_branch);
    let args = vec!["rebase".to_string(), target.clone()];

    execute_pty_command(model, args, format!("Rebase onto {}", target))
}

/// Rebase the current branch onto its configured upstream branch.
/// The upstream is read from the rebase popup state.
fn onto_upstream(model: &mut Model) -> Option<Message> {
    let upstream =
        if let Some(PopupContent::Command(PopupContentCommand::Rebase(ref state))) = model.popup {
            state.upstream.clone()
        } else {
            return None;
        }?;

    let args = vec!["rebase".to_string(), upstream.clone()];
    execute_pty_command(model, args, format!("Rebase onto {}", upstream))
}

/// Rebase the current branch onto the given remote branch (e.g. "origin/main"),
/// setting it as the upstream first.
fn onto_upstream_setting(model: &mut Model, upstream: String) -> Option<Message> {
    if let Err(e) = set_upstream_branch(&model.git_info.repository, &upstream) {
        model.popup = Some(PopupContent::Error {
            message: format!("Failed to set upstream: {}", e),
        });
        return None;
    }

    let args = vec!["rebase".to_string(), upstream.clone()];
    execute_pty_command(model, args, format!("Rebase onto {}", upstream))
}

const TOAST_DURATION: Duration = Duration::from_secs(5);

fn continue_rebase(model: &mut Model) -> Option<Message> {
    model.popup = None;
    match rebase::run_rebase_continue_with_editor(&model.workdir) {
        Ok(CommitResult { success, message }) => {
            model.toast = Some(Toast {
                message,
                style: if success {
                    ToastStyle::Success
                } else {
                    ToastStyle::Warning
                },
                expires_at: Instant::now() + TOAST_DURATION,
            });
        }
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: e.to_string(),
            });
        }
    }
    Some(Message::Refresh)
}

fn skip_rebase(model: &mut Model) -> Option<Message> {
    execute_pty_command(
        model,
        vec!["rebase".to_string(), "--skip".to_string()],
        "Rebase".to_string(),
    )
}

fn abort_rebase(model: &mut Model) -> Option<Message> {
    execute_pty_command(
        model,
        vec!["rebase".to_string(), "--abort".to_string()],
        "Rebase".to_string(),
    )
}
