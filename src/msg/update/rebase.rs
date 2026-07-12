use std::time::{Duration, Instant};

use crate::{
    git::rebase::{self, CommitResult},
    model::{Model, Toast, ToastStyle, ViewMode, popup::PopupContent},
    msg::{Message, RebaseCommand, update::pty_helper::execute_pty_command},
};

pub fn update(model: &mut Model, rebase_command: RebaseCommand) -> Option<Message> {
    match rebase_command {
        RebaseCommand::Elsewhere(target) => elsewhere(model, target),
        RebaseCommand::ExecuteInteractive => execute_interactive(model),
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
