use std::time::{Duration, Instant};

use crate::{
    git::revert::{self, CommitResult},
    model::{Model, Toast, ToastStyle, popup::PopupContent},
    msg::{Message, RevertCommand, update::pty_helper::execute_pty_command},
};

const TOAST_DURATION: Duration = Duration::from_secs(5);

pub fn update(model: &mut Model, cmd: RevertCommand) -> Option<Message> {
    match cmd {
        RevertCommand::Commits(hashes) => commits(model, hashes),
        RevertCommand::NoCommit(hashes) => no_commit(model, hashes),
        RevertCommand::Continue => continue_revert(model),
        RevertCommand::Skip => skip_revert(model),
        RevertCommand::Abort => abort_revert(model),
    }
}

fn commits(model: &mut Model, hashes: Vec<String>) -> Option<Message> {
    if hashes.is_empty() {
        return None;
    }
    let mut args = vec!["revert".to_string(), "--no-edit".to_string()];
    args.extend(hashes);
    execute_pty_command(model, args, "Revert".to_string())
}

fn no_commit(model: &mut Model, hashes: Vec<String>) -> Option<Message> {
    if hashes.is_empty() {
        return None;
    }
    let mut args = vec!["revert".to_string(), "--no-commit".to_string()];
    args.extend(hashes);
    execute_pty_command(model, args, "Revert".to_string())
}

fn continue_revert(model: &mut Model) -> Option<Message> {
    model.popup = None;
    match revert::run_revert_continue_with_editor(&model.workdir) {
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

fn skip_revert(model: &mut Model) -> Option<Message> {
    execute_pty_command(
        model,
        vec!["revert".to_string(), "--skip".to_string()],
        "Revert".to_string(),
    )
}

fn abort_revert(model: &mut Model) -> Option<Message> {
    execute_pty_command(
        model,
        vec!["revert".to_string(), "--abort".to_string()],
        "Revert".to_string(),
    )
}
