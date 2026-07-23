use std::time::Instant;

use crate::{
    model::{
        Model, Toast, ToastStyle, ViewMode,
        popup::{ConfirmAction, ConfirmPopupState, PopupContent},
    },
    msg::{DiscardSource, Message, ReverseTarget, update::commit::TOAST_DURATION},
};

use super::{
    apply_selected::build_preview_patch,
    discard_selected::determine_source,
    selection::{
        Selection, SelectionContext, get_normal_mode_selection, get_visual_mode_selection,
    },
};

/// Reverses the change under the cursor (or visual selection) in the working
/// tree, after confirmation. Mirrors magit-reverse: in a commit/stash preview
/// the shown diff is reverse-applied; in the status view only staged changes
/// can be reversed (they stay staged, only the working tree is undone), while
/// unstaged and untracked changes are rejected.
pub fn update(model: &mut Model) -> Option<Message> {
    if model.view_mode == ViewMode::Preview {
        reverse_from_preview(model)
    } else {
        reverse_from_status(model)
    }
}

fn reverse_from_preview(model: &mut Model) -> Option<Message> {
    let (start, end) = if model.ui_model.is_visual_mode() {
        model.ui_model.visual_selection_range()?
    } else {
        (
            model.ui_model.cursor_position,
            model.ui_model.cursor_position,
        )
    };

    let patch = build_preview_patch(&model.ui_model.lines, start, end)?;

    let message = format_patch_message(&patch);
    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message,
        on_confirm: ConfirmAction::Reverse(ReverseTarget::Patch { patch }),
    }));
    None
}

fn reverse_from_status(model: &mut Model) -> Option<Message> {
    let cursor_pos = model.ui_model.cursor_position;
    let current_line = model.ui_model.lines.get(cursor_pos)?;
    let source = determine_source(current_line.section.as_ref())?;

    // Only committed or staged changes can be reversed; discard is the tool
    // for uncommitted ones. Mirror magit's error message.
    if source != DiscardSource::Staged {
        model.toast = Some(Toast {
            message: "Cannot reverse uncommitted changes".to_string(),
            style: ToastStyle::Warning,
            expires_at: Instant::now() + TOAST_DURATION,
        });
        return None;
    }

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
            cursor_pos,
            SelectionContext::Unstageable,
        )
    };

    let target = match selection {
        Selection::None => return None,
        Selection::Files(files) => ReverseTarget::Files {
            paths: files.into_iter().map(String::from).collect(),
        },
        Selection::Hunk { path, hunk_index } => ReverseTarget::Hunk {
            path: path.to_string(),
            hunk_index,
        },
        Selection::Hunks { path, hunk_indices } => ReverseTarget::Hunks {
            path: path.to_string(),
            hunk_indices,
        },
        Selection::Lines {
            path,
            hunk_index,
            line_indices,
        } => ReverseTarget::Lines {
            path: path.to_string(),
            hunk_index,
            line_indices,
        },
    };

    let message = format_reverse_message(&target);
    model.popup = Some(PopupContent::Confirm(ConfirmPopupState {
        message,
        on_confirm: ConfirmAction::Reverse(target),
    }));
    None
}

fn format_reverse_message(target: &ReverseTarget) -> String {
    match target {
        ReverseTarget::Patch { patch } => format_patch_message(patch),
        ReverseTarget::Files { paths } if paths.len() == 1 => {
            format!("Reverse staged changes in {}?", paths[0])
        }
        ReverseTarget::Files { paths } => {
            format!("Reverse staged changes in {} files?", paths.len())
        }
        ReverseTarget::Hunk { path, .. } => {
            format!("Reverse staged hunk in {}?", path)
        }
        ReverseTarget::Hunks {
            path, hunk_indices, ..
        } => {
            format!("Reverse {} staged hunks in {}?", hunk_indices.len(), path)
        }
        ReverseTarget::Lines {
            path, line_indices, ..
        } => {
            format!("Reverse {} staged lines in {}?", line_indices.len(), path)
        }
    }
}

/// Describes a preview patch for the confirmation prompt by counting its
/// files and hunks.
fn format_patch_message(patch: &str) -> String {
    let file_count = patch
        .lines()
        .filter(|l| l.starts_with("diff --git "))
        .count();
    let hunk_count = patch.lines().filter(|l| l.starts_with("@@")).count();

    match (file_count, hunk_count) {
        (n, _) if n > 1 => format!("Reverse changes in {} files?", n),
        (_, 1) => match patch_file_name(patch) {
            Some(path) => format!("Reverse hunk in {}?", path),
            None => "Reverse this change in the working tree?".to_string(),
        },
        (_, n) if n > 1 => match patch_file_name(patch) {
            Some(path) => format!("Reverse {} hunks in {}?", n, path),
            None => "Reverse these changes in the working tree?".to_string(),
        },
        _ => "Reverse this change in the working tree?".to_string(),
    }
}

/// Extracts the file name from the `---`/`+++` header lines of a patch.
fn patch_file_name(patch: &str) -> Option<&str> {
    patch.lines().find_map(|l| {
        l.strip_prefix("+++ b/")
            .or_else(|| l.strip_prefix("--- a/"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SINGLE_HUNK_PATCH: &str = "\
diff --git a/one.txt b/one.txt
index 1111111..2222222 100644
--- a/one.txt
+++ b/one.txt
@@ -1,2 +1,3 @@
 first
+added one
 second
";

    #[test]
    fn test_format_patch_message_single_hunk() {
        assert_eq!(
            format_patch_message(SINGLE_HUNK_PATCH),
            "Reverse hunk in one.txt?"
        );
    }

    #[test]
    fn test_format_patch_message_multiple_hunks_same_file() {
        let patch = format!(
            "{}@@ -10,2 +11,3 @@\n tenth\n+added two\n",
            SINGLE_HUNK_PATCH
        );
        assert_eq!(format_patch_message(&patch), "Reverse 2 hunks in one.txt?");
    }

    #[test]
    fn test_format_patch_message_multiple_files() {
        let patch = format!(
            "{}diff --git a/two.txt b/two.txt\n--- a/two.txt\n+++ b/two.txt\n@@ -1,1 +1,2 @@\n alpha\n+beta\n",
            SINGLE_HUNK_PATCH
        );
        assert_eq!(format_patch_message(&patch), "Reverse changes in 2 files?");
    }
}
