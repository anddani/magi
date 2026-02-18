use crate::{
    model::{
        Model, SectionType,
        popup::{ConfirmAction, ConfirmPopupState, PopupContent},
    },
    msg::{DiscardSource, DiscardTarget, Message},
};

use super::selection::{
    Selection, SelectionContext, get_normal_mode_selection, get_visual_mode_selection,
};

pub fn update(model: &mut Model) -> Option<Message> {
    // Get current cursor position to determine source
    let cursor_pos = model.ui_model.cursor_position;
    let current_line = model.ui_model.lines.get(cursor_pos)?;

    // Determine if we're in staged or unstaged section
    let source = determine_source(current_line.section.as_ref())?;

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
            cursor_pos,
            SelectionContext::Discardable,
        )
    };

    // Convert Selection to DiscardTarget (with owned strings and source)
    let target = match selection {
        Selection::None => return None,
        Selection::Files(files) => DiscardTarget::Files {
            paths: files.into_iter().map(String::from).collect(),
            source,
        },
        Selection::Hunk { path, hunk_index } => DiscardTarget::Hunk {
            path: path.to_string(),
            hunk_index,
            source,
        },
        Selection::Hunks { path, hunk_indices } => DiscardTarget::Hunks {
            path: path.to_string(),
            hunk_indices,
            source,
        },
        Selection::Lines {
            path,
            hunk_index,
            line_indices,
        } => DiscardTarget::Lines {
            path: path.to_string(),
            hunk_index,
            line_indices,
            source,
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

/// Determines the discard source (staged vs unstaged) from a section type.
fn determine_source(section: Option<&SectionType>) -> Option<DiscardSource> {
    match section {
        Some(SectionType::UnstagedChanges)
        | Some(SectionType::UnstagedFile { .. })
        | Some(SectionType::UnstagedHunk { .. }) => Some(DiscardSource::Unstaged),
        Some(SectionType::StagedChanges)
        | Some(SectionType::StagedFile { .. })
        | Some(SectionType::StagedHunk { .. }) => Some(DiscardSource::Staged),
        _ => None,
    }
}

fn format_discard_message(target: &DiscardTarget) -> String {
    let staged_prefix = match target {
        DiscardTarget::Files { source, .. }
        | DiscardTarget::Hunk { source, .. }
        | DiscardTarget::Hunks { source, .. }
        | DiscardTarget::Lines { source, .. } => {
            if *source == DiscardSource::Staged {
                "staged "
            } else {
                ""
            }
        }
    };

    match target {
        DiscardTarget::Files { paths, .. } if paths.len() == 1 => {
            format!("Discard {}changes in {}?", staged_prefix, paths[0])
        }
        DiscardTarget::Files { paths, .. } => {
            format!("Discard {}changes in {} files?", staged_prefix, paths.len())
        }
        DiscardTarget::Hunk { path, .. } => {
            format!("Discard {}hunk in {}?", staged_prefix, path)
        }
        DiscardTarget::Hunks {
            path, hunk_indices, ..
        } => {
            format!(
                "Discard {} {}hunks in {}?",
                hunk_indices.len(),
                staged_prefix,
                path
            )
        }
        DiscardTarget::Lines {
            path, line_indices, ..
        } => {
            format!(
                "Discard {} {}lines in {}?",
                line_indices.len(),
                staged_prefix,
                path
            )
        }
    }
}
