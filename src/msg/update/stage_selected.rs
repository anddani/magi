use crate::{
    git::stage::{stage_files, stage_hunk, stage_lines},
    model::{Model, popup::PopupContent},
    msg::Message,
};

use super::selection::{
    Selection, SelectionContext, get_normal_mode_selection, get_visual_mode_selection,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let repo_path = model.workdir.clone();

    let selection = if model.ui_model.is_visual_mode() {
        let (start, end) = model.ui_model.visual_selection_range()?;
        get_visual_mode_selection(
            &model.ui_model.lines,
            start,
            end,
            &model.ui_model.collapsed_sections,
            SelectionContext::Stageable,
        )
    } else {
        get_normal_mode_selection(
            &model.ui_model.lines,
            model.ui_model.cursor_position,
            SelectionContext::Stageable,
        )
    };

    // Exit visual mode after staging
    model.ui_model.visual_mode_anchor = None;

    let result = apply_selection(&repo_path, selection);

    if let Err(e) = result {
        model.popup = Some(PopupContent::Error {
            message: format!("Error staging: {}", e),
        });
    }

    Some(Message::Refresh)
}

fn apply_selection(
    repo_path: &std::path::Path,
    selection: Selection,
) -> Result<(), crate::errors::MagiError> {
    match selection {
        Selection::None => Ok(()),
        Selection::Files(files) => stage_files(repo_path, &files),
        Selection::Hunk { path, hunk_index } => stage_hunk(repo_path, path, hunk_index),
        Selection::Hunks { path, hunk_indices } => {
            // Apply hunks in reverse order (highest index first) to avoid index shifts
            for idx in hunk_indices {
                stage_hunk(repo_path, path, idx)?;
            }
            Ok(())
        }
        Selection::Lines {
            path,
            hunk_index,
            line_indices,
        } => stage_lines(repo_path, path, hunk_index, &line_indices),
    }
}
