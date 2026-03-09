use std::time::Instant;

use crate::{
    git::{commit::CommitResult, merge},
    model::{Model, PopupContent, Toast, ToastStyle},
    msg::{
        MergeCommand, Message,
        update::{commit::TOAST_DURATION, pty_helper::execute_pty_command},
    },
};

pub fn update(model: &mut Model, cmd: MergeCommand) -> Option<Message> {
    match cmd {
        MergeCommand::Branch(branch) => merge_branch(model, branch),
        MergeCommand::Continue => continue_merge(model),
        MergeCommand::Abort => abort_merge(model),
    }
}

fn merge_branch(model: &mut Model, branch: String) -> Option<Message> {
    model.popup = None;
    execute_pty_command(
        model,
        vec!["merge".to_string(), branch],
        "Merge".to_string(),
    )
}

fn continue_merge(model: &mut Model) -> Option<Message> {
    model.popup = None;
    match merge::run_merge_continue_with_editor(&model.workdir) {
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

fn abort_merge(model: &mut Model) -> Option<Message> {
    model.popup = None;
    execute_pty_command(
        model,
        vec!["merge".to_string(), "--abort".to_string()],
        "Merge".to_string(),
    )
}
