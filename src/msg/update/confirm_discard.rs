use crate::{
    git::discard::{discard_files, discard_hunk, discard_lines},
    model::{Model, popup::PopupContent},
    msg::{DiscardTarget, Message},
};

pub fn update(model: &mut Model, target: DiscardTarget) -> Option<Message> {
    // Clear visual mode and popup
    model.ui_model.visual_mode_anchor = None;
    model.popup = None;

    let repo_path = model.workdir.clone();
    let result = apply_discard(&repo_path, target);

    if let Err(e) = result {
        model.popup = Some(PopupContent::Error {
            message: format!("Error discarding: {}", e),
        });
    }

    Some(Message::Refresh)
}

fn apply_discard(
    repo_path: &std::path::Path,
    target: DiscardTarget,
) -> Result<(), crate::errors::MagiError> {
    match target {
        DiscardTarget::Files(files) => {
            let file_refs: Vec<&str> = files.iter().map(String::as_str).collect();
            discard_files(repo_path, &file_refs)
        }
        DiscardTarget::Hunk { path, hunk_index } => discard_hunk(repo_path, &path, hunk_index),
        DiscardTarget::Hunks { path, hunk_indices } => {
            // Apply hunks in reverse order (highest index first) to avoid index shifts
            for idx in hunk_indices {
                discard_hunk(repo_path, &path, idx)?;
            }
            Ok(())
        }
        DiscardTarget::Lines {
            path,
            hunk_index,
            line_indices,
        } => discard_lines(repo_path, &path, hunk_index, &line_indices),
    }
}
