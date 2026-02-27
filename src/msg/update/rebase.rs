use std::time::{Duration, Instant};

use crate::{
    git::rebase::{self, CommitResult},
    model::{Model, Toast, ToastStyle, popup::PopupContent},
    msg::{Message, RebaseCommand, update::pty_helper::execute_pty_command},
};

pub fn update(model: &mut Model, rebase_command: RebaseCommand) -> Option<Message> {
    match rebase_command {
        RebaseCommand::Elsewhere(target) => elsewhere(model, target),
        RebaseCommand::Continue => continue_rebase(model),
        RebaseCommand::Skip => skip_rebase(model),
        RebaseCommand::Abort => abort_rebase(model),
    }
}

fn elsewhere(model: &mut Model, target: String) -> Option<Message> {
    let args = vec!["rebase".to_string(), target];
    execute_pty_command(model, args, "Rebase".to_string())
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
