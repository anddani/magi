use crate::{
    git::stage::{unstage_files, unstage_hunk, unstage_lines},
    model::{Model, cursor_context::CursorContext, popup::PopupContent},
    msg::Message,
};

use super::selection::{
    Selection, SelectionContext, get_normal_mode_selection, get_visual_mode_selection,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let repo_path = model.workdir.clone();

    // Capture cursor context before unstaging (to be used by refresh)
    model.cursor_reposition_context = Some(CursorContext::capture(
        &model.ui_model.lines,
        model.ui_model.cursor_position,
    ));

    let selection = if model.ui_model.is_visual_mode() {
        let (start, end) = model.ui_model.visual_selection_range()?;
        get_visual_mode_selection(
            &model.ui_model.lines,
            start,
            end,
            &model.ui_model.collapsed_sections,
            SelectionContext::Unstageable,
        )
    } else {
        get_normal_mode_selection(
            &model.ui_model.lines,
            model.ui_model.cursor_position,
            SelectionContext::Unstageable,
        )
    };

    // Exit visual mode after unstaging
    model.ui_model.visual_mode_anchor = None;

    let result = apply_selection(&repo_path, selection);

    if let Err(e) = result {
        model.popup = Some(PopupContent::Error {
            message: format!("Error unstaging: {}", e),
        });
        // Clear cursor context on error
        model.cursor_reposition_context = None;
        return None;
    }

    Some(Message::Refresh)
}

fn apply_selection(
    repo_path: &std::path::Path,
    selection: Selection,
) -> Result<(), crate::errors::MagiError> {
    match selection {
        Selection::None => Ok(()),
        Selection::Files(files) => unstage_files(repo_path, &files),
        Selection::Hunk { path, hunk_index } => unstage_hunk(repo_path, path, hunk_index),
        Selection::Hunks { path, hunk_indices } => {
            // Apply hunks in reverse order (highest index first) to avoid index shifts
            for idx in hunk_indices {
                unstage_hunk(repo_path, path, idx)?;
            }
            Ok(())
        }
        Selection::Lines {
            path,
            hunk_index,
            line_indices,
        } => unstage_lines(repo_path, path, hunk_index, &line_indices),
    }
}
