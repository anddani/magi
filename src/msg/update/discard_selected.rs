use crate::{
    model::{
        Model,
        popup::{ConfirmAction, ConfirmPopupState, PopupContent},
    },
    msg::{DiscardTarget, Message},
};

use super::selection::{
    Selection, SelectionContext, get_normal_mode_selection, get_visual_mode_selection,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let selection = if model.ui_model.is_visual_mode() {
        let (start, end) = model.ui_model.visual_selection_range()?;
        get_visual_mode_selection(
            &model.ui_model.lines,
            start,
            end,
            &model.ui_model.collapsed_sections,
            SelectionContext::Discardable,
        )
    } else {
        get_normal_mode_selection(
            &model.ui_model.lines,
            model.ui_model.cursor_position,
            SelectionContext::Discardable,
        )
    };

    // Convert Selection to DiscardTarget (with owned strings)
    let target = match selection {
        Selection::None => return None,
        Selection::Files(files) => {
            DiscardTarget::Files(files.into_iter().map(String::from).collect())
        }
        Selection::Hunk { path, hunk_index } => DiscardTarget::Hunk {
            path: path.to_string(),
            hunk_index,
        },
        Selection::Hunks { path, hunk_indices } => DiscardTarget::Hunks {
            path: path.to_string(),
            hunk_indices,
        },
        Selection::Lines {
            path,
            hunk_index,
            line_indices,
        } => DiscardTarget::Lines {
            path: path.to_string(),
            hunk_index,
            line_indices,
        },
    };

    // Show confirmation popup
    let message = format_discard_message(&target);
    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message,
        on_confirm: ConfirmAction::DiscardChanges(target),
    }));

    None
}

fn format_discard_message(target: &DiscardTarget) -> String {
    match target {
        DiscardTarget::Files(files) if files.len() == 1 => {
            format!("Discard changes in {}?", files[0])
        }
        DiscardTarget::Files(files) => {
            format!("Discard changes in {} files?", files.len())
        }
        DiscardTarget::Hunk { path, .. } => {
            format!("Discard hunk in {}?", path)
        }
        DiscardTarget::Hunks { path, hunk_indices } => {
            format!("Discard {} hunks in {}?", hunk_indices.len(), path)
        }
        DiscardTarget::Lines {
            path, line_indices, ..
        } => {
            format!("Discard {} lines in {}?", line_indices.len(), path)
        }
    }
}
