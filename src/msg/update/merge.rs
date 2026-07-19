use std::time::Instant;

use crate::{
    git::{commit::CommitResult, merge, preview},
    model::{Model, PopupContent, Toast, ToastStyle, ViewMode},
    msg::{
        MergeCommand, Message,
        update::{
            commit::TOAST_DURATION, pty_helper::execute_pty_command,
            show_merge_popup::merge_in_progress,
        },
    },
};

pub fn update(model: &mut Model, cmd: MergeCommand) -> Option<Message> {
    match cmd {
        MergeCommand::Branch(branch) => merge_branch(model, branch, false),
        MergeCommand::EditMessage(branch) => merge_branch(model, branch, true),
        MergeCommand::NoCommit(branch) => merge_no_commit(model, branch),
        MergeCommand::Absorb(branch) => absorb_branch(model, branch),
        MergeCommand::Preview(branch) => preview_merge(model, branch),
        MergeCommand::Squash(branch) => squash_merge(model, branch),
        MergeCommand::Continue => continue_merge(model),
        MergeCommand::Abort => abort_merge(model),
    }
}

fn merge_branch(model: &mut Model, branch: String, edit_message: bool) -> Option<Message> {
    model.popup = None;
    let result = if edit_message {
        merge::run_merge_edit_with_editor(&model.workdir, &branch)
    } else {
        merge::run_merge_with_editor(&model.workdir, &branch)
    };
    match result {
        Ok(CommitResult { success, message }) => {
            if let Some(conflicts) = (!success).then(|| unresolved_conflicts(model)).flatten() {
                model.popup = Some(PopupContent::Error {
                    message: conflict_message(
                        &format!("Merge of '{}' stopped due to conflicts:", branch),
                        &conflicts,
                    ),
                });
            } else {
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
        }
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: e.to_string(),
            });
        }
    }
    Some(Message::Refresh)
}

/// Returns the conflicted paths when a merge is paused on unresolved
/// conflicts, or `None` when the failure was not conflict-related.
fn unresolved_conflicts(model: &Model) -> Option<Vec<String>> {
    if !merge_in_progress(&model.workdir) {
        return None;
    }
    let conflicts = merge::conflicted_files(&model.workdir).unwrap_or_default();
    (!conflicts.is_empty()).then_some(conflicts)
}

fn conflict_message(headline: &str, conflicts: &[String]) -> String {
    let mut message = format!("{}\n", headline);
    for path in conflicts {
        message.push_str(&format!("\n  {}", path));
    }
    message.push_str("\n\nResolve the conflicts, then continue or abort from the merge popup (m).");
    message
}

fn absorb_branch(model: &mut Model, branch: String) -> Option<Message> {
    model.popup = None;
    match merge::run_merge_absorb(&model.workdir, &branch) {
        Ok(CommitResult { success, message }) => {
            if let Some(conflicts) = (!success).then(|| unresolved_conflicts(model)).flatten() {
                model.popup = Some(PopupContent::Error {
                    message: conflict_message(
                        &format!("Absorb of '{}' stopped due to conflicts:", branch),
                        &conflicts,
                    ),
                });
            } else {
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
        }
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: e.to_string(),
            });
        }
    }
    Some(Message::Refresh)
}

fn preview_merge(model: &mut Model, branch: String) -> Option<Message> {
    model.popup = None;
    let head_name = model
        .git_info
        .current_branch()
        .unwrap_or_else(|| "HEAD".to_string());
    match preview::get_merge_preview_lines(&model.workdir, &branch, &head_name) {
        Ok(lines) => {
            model.preview_return_mode = Some(model.view_mode.clone());
            model.preview_return_ui_model = Some(model.ui_model.clone());
            model.ui_model.lines = lines;
            model.ui_model.cursor_position = 0;
            model.ui_model.scroll_offset = 0;
            model.view_mode = ViewMode::Preview;
        }
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: e.to_string(),
            });
        }
    }
    None
}

fn merge_no_commit(model: &mut Model, branch: String) -> Option<Message> {
    execute_pty_command(
        model,
        vec![
            "merge".to_string(),
            "--no-commit".to_string(),
            "--no-ff".to_string(),
            branch,
        ],
        "Merge".to_string(),
    )
}

fn squash_merge(model: &mut Model, branch: String) -> Option<Message> {
    execute_pty_command(
        model,
        vec!["merge".to_string(), "--squash".to_string(), branch],
        "Merge".to_string(),
    )
}

fn continue_merge(model: &mut Model) -> Option<Message> {
    model.popup = None;
    match merge::run_merge_continue_with_editor(&model.workdir) {
        Ok(CommitResult { success, message }) => {
            if let Some(conflicts) = (!success).then(|| unresolved_conflicts(model)).flatten() {
                model.popup = Some(PopupContent::Error {
                    message: conflict_message(
                        "Cannot continue the merge, there are unresolved conflicts:",
                        &conflicts,
                    ),
                });
                return Some(Message::Refresh);
            }
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
