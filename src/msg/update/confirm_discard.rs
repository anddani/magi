use crate::{
    git::discard::{
        discard_files, discard_hunk, discard_lines, discard_staged_files, discard_staged_hunk,
        discard_staged_lines,
    },
    model::{Model, popup::PopupContent},
    msg::{DiscardSource, DiscardTarget, Message},
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
        DiscardTarget::Files { paths, source } => {
            let file_refs: Vec<&str> = paths.iter().map(String::as_str).collect();
            match source {
                DiscardSource::Unstaged => discard_files(repo_path, &file_refs),
                DiscardSource::Staged => discard_staged_files(repo_path, &file_refs),
            }
        }
        DiscardTarget::Hunk {
            path,
            hunk_index,
            source,
        } => match source {
            DiscardSource::Unstaged => discard_hunk(repo_path, &path, hunk_index),
            DiscardSource::Staged => discard_staged_hunk(repo_path, &path, hunk_index),
        },
        DiscardTarget::Hunks {
            path,
            hunk_indices,
            source,
        } => {
            // Apply hunks in reverse order (highest index first) to avoid index shifts
            for idx in hunk_indices {
                match source {
                    DiscardSource::Unstaged => discard_hunk(repo_path, &path, idx)?,
                    DiscardSource::Staged => discard_staged_hunk(repo_path, &path, idx)?,
                }
            }
            Ok(())
        }
        DiscardTarget::Lines {
            path,
            hunk_index,
            line_indices,
            source,
        } => match source {
            DiscardSource::Unstaged => discard_lines(repo_path, &path, hunk_index, &line_indices),
            DiscardSource::Staged => {
                discard_staged_lines(repo_path, &path, hunk_index, &line_indices)
            }
        },
    }
}
