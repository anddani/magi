use std::time::Instant;

use crate::{
    git::apply::apply_patch,
    model::{
        Line, LineContent, Model, PreviewLineType, Toast, ToastStyle, ViewMode, popup::PopupContent,
    },
    msg::{Message, update::commit::TOAST_DURATION},
};

use super::selection::{
    Selection, SelectionContext, get_normal_mode_selection, get_visual_mode_selection,
};

pub fn update(model: &mut Model) -> Option<Message> {
    if model.view_mode == ViewMode::Preview {
        apply_from_preview(model)
    } else {
        reject_worktree_change(model)
    }
}

/// Outside of Preview mode every diff shown already describes the working
/// tree (or the index), so there is nothing to apply. Mirror magit's error
/// message when the cursor is on such a change.
fn reject_worktree_change(model: &mut Model) -> Option<Message> {
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

    if !matches!(selection, Selection::None) {
        model.toast = Some(Toast {
            message: "Change is already in the working tree".to_string(),
            style: ToastStyle::Warning,
            expires_at: Instant::now() + TOAST_DURATION,
        });
    }
    None
}

/// Applies the hunk under the cursor (or the hunks covered by the visual
/// selection) of a commit/stash preview to the working tree. When the cursor
/// is on a file header, the whole diff of that file is applied.
fn apply_from_preview(model: &mut Model) -> Option<Message> {
    let (start, end) = if model.ui_model.is_visual_mode() {
        model.ui_model.visual_selection_range()?
    } else {
        (
            model.ui_model.cursor_position,
            model.ui_model.cursor_position,
        )
    };
    model.ui_model.visual_mode_anchor = None;

    let patch = build_preview_patch(&model.ui_model.lines, start, end)?;

    match apply_patch(&model.workdir, &patch) {
        Ok(()) => {
            model.toast = Some(Toast {
                message: "Changes applied to the working tree".to_string(),
                style: ToastStyle::Success,
                expires_at: Instant::now() + TOAST_DURATION,
            });
        }
        Err(e) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Error applying: {}", e),
            });
        }
    }
    None
}

/// The line ranges of one file's diff within the preview. Ranges are
/// half-open (`start..end`) indices into the preview lines.
struct FileBlock {
    /// The file header (`diff --git` through `+++`/mode lines)
    header: (usize, usize),
    /// One range per hunk, each starting at its `@@` header
    hunks: Vec<(usize, usize)>,
}

/// Builds a unified diff from the preview lines covered by the inclusive
/// selection `[start, end]`. A hunk is included when the selection touches
/// any of its lines; selecting a file header includes that file's whole diff.
/// Returns `None` when the selection touches no diff content (e.g. only the
/// commit metadata).
fn build_preview_patch(lines: &[Line], start: usize, end: usize) -> Option<String> {
    let mut blocks: Vec<FileBlock> = Vec::new();
    let mut hunk_start: Option<usize> = None;

    for (i, line) in lines.iter().enumerate() {
        let LineContent::PreviewLine { content, line_type } = &line.content else {
            continue;
        };

        if matches!(line_type, PreviewLineType::DiffFileHeader)
            && content.starts_with("diff --git ")
        {
            if let (Some(hunk), Some(block)) = (hunk_start.take(), blocks.last_mut()) {
                block.hunks.push((hunk, i));
            }
            blocks.push(FileBlock {
                header: (i, i + 1),
                hunks: Vec::new(),
            });
            continue;
        }

        let Some(block) = blocks.last_mut() else {
            continue;
        };
        if matches!(line_type, PreviewLineType::HunkHeader) {
            if let Some(hunk) = hunk_start.take() {
                block.hunks.push((hunk, i));
            }
            hunk_start = Some(i);
        } else if hunk_start.is_none() {
            // Still in the file header (index, ---, +++, mode lines)
            block.header.1 = i + 1;
        }
    }
    if let (Some(hunk), Some(block)) = (hunk_start, blocks.last_mut()) {
        block.hunks.push((hunk, lines.len()));
    }

    // Selection [start, end] (inclusive) overlaps range [s, e) (half-open)
    let intersects = |(s, e): (usize, usize)| s <= end && e > start;

    let mut patch = String::new();
    for block in &blocks {
        let header_selected = intersects(block.header);
        let selected_hunks: Vec<(usize, usize)> = if header_selected {
            block.hunks.clone()
        } else {
            block
                .hunks
                .iter()
                .copied()
                .filter(|range| intersects(*range))
                .collect()
        };

        if selected_hunks.is_empty() && !header_selected {
            continue;
        }

        let ranges = std::iter::once(block.header).chain(selected_hunks);
        for (s, e) in ranges {
            for line in &lines[s..e] {
                if let LineContent::PreviewLine { content, .. } = &line.content {
                    patch.push_str(content);
                    patch.push('\n');
                }
            }
        }
    }

    if patch.is_empty() { None } else { Some(patch) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::preview::parse_preview_output;

    const PREVIEW: &str = "\
commit abc1234
Author: Test <test@example.com>
Date:   Mon Jan 1 00:00:00 2024

    Change two files

diff --git a/one.txt b/one.txt
index 1111111..2222222 100644
--- a/one.txt
+++ b/one.txt
@@ -1,2 +1,3 @@
 first
+added one
 second
@@ -10,2 +11,3 @@
 tenth
+added two
 eleventh
diff --git a/two.txt b/two.txt
index 3333333..4444444 100644
--- a/two.txt
+++ b/two.txt
@@ -1,1 +1,2 @@
 alpha
+beta
";

    fn preview_lines() -> Vec<Line> {
        parse_preview_output(PREVIEW)
    }

    fn line_index(lines: &[Line], text: &str) -> usize {
        lines
            .iter()
            .position(|l| matches!(&l.content, LineContent::PreviewLine { content, .. } if content == text))
            .unwrap()
    }

    #[test]
    fn test_cursor_on_commit_metadata_returns_none() {
        let lines = preview_lines();
        assert!(build_preview_patch(&lines, 0, 0).is_none());
    }

    #[test]
    fn test_cursor_on_hunk_builds_single_hunk_patch() {
        let lines = preview_lines();
        let pos = line_index(&lines, "@@ -1,2 +1,3 @@");

        let patch = build_preview_patch(&lines, pos, pos).unwrap();

        assert!(patch.starts_with("diff --git a/one.txt b/one.txt\n"));
        assert!(patch.contains("+added one"));
        assert!(!patch.contains("+added two"));
        assert!(!patch.contains("two.txt"));
    }

    #[test]
    fn test_cursor_on_diff_line_builds_containing_hunk_patch() {
        let lines = preview_lines();
        let pos = line_index(&lines, "+added two");

        let patch = build_preview_patch(&lines, pos, pos).unwrap();

        assert!(patch.contains("@@ -10,2 +11,3 @@"));
        assert!(patch.contains("+added two"));
        assert!(!patch.contains("+added one"));
    }

    #[test]
    fn test_cursor_on_file_header_includes_all_hunks_of_file() {
        let lines = preview_lines();
        let pos = line_index(&lines, "diff --git a/one.txt b/one.txt");

        let patch = build_preview_patch(&lines, pos, pos).unwrap();

        assert!(patch.contains("+added one"));
        assert!(patch.contains("+added two"));
        assert!(!patch.contains("two.txt"));
    }

    #[test]
    fn test_selection_spanning_files_includes_both_headers() {
        let lines = preview_lines();
        let start = line_index(&lines, "+added two");
        let end = line_index(&lines, "+beta");

        let patch = build_preview_patch(&lines, start, end).unwrap();

        assert!(patch.contains("diff --git a/one.txt b/one.txt"));
        assert!(patch.contains("diff --git a/two.txt b/two.txt"));
        assert!(patch.contains("+added two"));
        assert!(patch.contains("+beta"));
        // The first hunk of one.txt was not touched by the selection
        assert!(!patch.contains("+added one"));
    }

    #[test]
    fn test_last_hunk_extends_to_end_of_preview() {
        let lines = preview_lines();
        let pos = line_index(&lines, "+beta");

        let patch = build_preview_patch(&lines, pos, pos).unwrap();

        assert!(patch.ends_with("+beta\n"));
    }
}
