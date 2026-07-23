use std::time::Instant;

use crate::{
    git::reverse::{
        reverse_patch, reverse_staged_files, reverse_staged_hunk, reverse_staged_hunks,
        reverse_staged_lines,
    },
    model::{Model, Toast, ToastStyle, popup::PopupContent},
    msg::{Message, ReverseTarget, update::commit::TOAST_DURATION},
};

pub fn update(model: &mut Model, target: ReverseTarget) -> Option<Message> {
    // Clear visual mode and popup
    model.ui_model.visual_mode_anchor = None;
    model.popup = None;

    let repo_path = model.workdir.clone();
    match apply_reverse(&repo_path, target) {
        Ok(()) => {
            model.toast = Some(Toast {
                message: "Changes reversed in the working tree".to_string(),
                style: ToastStyle::Success,
                expires_at: Instant::now() + TOAST_DURATION,
            });
        }
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Error reversing: {}", e),
            });
        }
    }

    Some(Message::Refresh)
}

fn apply_reverse(
    repo_path: &std::path::Path,
    target: ReverseTarget,
) -> Result<(), crate::errors::MagiError> {
    match target {
        ReverseTarget::Patch { patch } => reverse_patch(repo_path, &patch),
        ReverseTarget::Files { paths } => {
            let file_refs: Vec<&str> = paths.iter().map(String::as_str).collect();
            reverse_staged_files(repo_path, &file_refs)
        }
        ReverseTarget::Hunk { path, hunk_index } => {
            reverse_staged_hunk(repo_path, &path, hunk_index)
        }
        ReverseTarget::Hunks { path, hunk_indices } => {
            reverse_staged_hunks(repo_path, &path, &hunk_indices)
        }
        ReverseTarget::Lines {
            path,
            hunk_index,
            line_indices,
        } => reverse_staged_lines(repo_path, &path, hunk_index, &line_indices),
    }
}
