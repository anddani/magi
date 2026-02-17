use crate::{
    git::stage::{unstage_files, unstage_hunk, unstage_lines},
    model::{LineContent, Model, SectionType, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let repo_path = model.workdir.clone();

    let result = if model.ui_model.is_visual_mode() {
        handle_visual_mode(model, &repo_path)
    } else {
        handle_normal_mode(model, &repo_path)
    };

    // Exit visual mode after unstaging
    model.ui_model.visual_mode_anchor = None;

    if let Err(e) = result {
        model.popup = Some(PopupContent::Error {
            message: format!("Error unstaging: {}", e),
        });
    }

    Some(Message::Refresh)
}

fn handle_normal_mode(
    model: &Model,
    repo_path: &std::path::Path,
) -> Result<(), crate::errors::MagiError> {
    let lines = &model.ui_model.lines;
    let pos = model.ui_model.cursor_position;
    if pos >= lines.len() {
        return Ok(());
    }

    let line = &lines[pos];

    match (&line.content, &line.section) {
        // Section header for staged changes -> unstage all staged
        (LineContent::SectionHeader { .. }, Some(SectionType::StagedChanges)) => {
            let files: Vec<&str> = lines
                .iter()
                .filter_map(|l| match &l.content {
                    LineContent::StagedFile(fc) => Some(fc.path.as_str()),
                    _ => None,
                })
                .collect();
            unstage_files(repo_path, &files)
        }
        // Staged file -> unstage that file
        (LineContent::StagedFile(fc), _) => unstage_files(repo_path, &[fc.path.as_str()]),
        // Diff hunk in staged section -> unstage that hunk
        (
            LineContent::DiffHunk(_),
            Some(SectionType::StagedHunk {
                path, hunk_index, ..
            }),
        ) => unstage_hunk(repo_path, path, *hunk_index),
        // Diff line in staged section -> unstage the whole hunk it belongs to
        (
            LineContent::DiffLine(_),
            Some(SectionType::StagedHunk {
                path, hunk_index, ..
            }),
        ) => unstage_hunk(repo_path, path, *hunk_index),
        _ => Ok(()),
    }
}

fn handle_visual_mode(
    model: &Model,
    repo_path: &std::path::Path,
) -> Result<(), crate::errors::MagiError> {
    let (start, end) = match model.ui_model.visual_selection_range() {
        Some(range) => range,
        None => return Ok(()),
    };

    let lines = &model.ui_model.lines;
    let end = end.min(lines.len().saturating_sub(1));

    // Collect meaningful lines in selection (skip empty and hidden/collapsed lines)
    let selected: Vec<(usize, &crate::model::Line)> = (start..=end)
        .filter_map(|i| {
            let line = &lines[i];
            if matches!(line.content, LineContent::EmptyLine)
                || line.is_hidden(&model.ui_model.collapsed_sections)
            {
                return None;
            }
            Some((i, line))
        })
        .collect();

    if selected.is_empty() {
        return Ok(());
    }

    // Check if all are staged files
    let all_staged_files = selected.iter().all(|(_, l)| {
        matches!(
            l.content,
            LineContent::StagedFile(_) | LineContent::SectionHeader { .. }
        )
    });
    if all_staged_files {
        let files: Vec<&str> = selected
            .iter()
            .filter_map(|(_, l)| match &l.content {
                LineContent::StagedFile(fc) => Some(fc.path.as_str()),
                _ => None,
            })
            .collect();
        if !files.is_empty() {
            return unstage_files(repo_path, &files);
        }
    }

    // Check if all are diff lines in the same hunk
    let all_diff_lines = selected
        .iter()
        .all(|(_, l)| matches!(l.content, LineContent::DiffLine(_)));
    if all_diff_lines {
        let sections: Vec<&SectionType> = selected
            .iter()
            .filter_map(|(_, l)| l.section.as_ref())
            .collect();

        if let Some(SectionType::StagedHunk { path, hunk_index }) = sections.first() {
            let all_same_hunk = sections.iter().all(|s| {
                matches!(s, SectionType::StagedHunk { path: p, hunk_index: h } if p == path && h == hunk_index)
            });

            if all_same_hunk {
                // Calculate which lines within the hunk are selected
                let line_indices = compute_hunk_line_indices(lines, start, end, path, *hunk_index);
                if !line_indices.is_empty() {
                    return unstage_lines(repo_path, path, *hunk_index, &line_indices);
                }
            }
        }
    }

    // Check if all are diff hunks (or diff lines within hunks) in the same file
    let all_hunks = selected.iter().all(|(_, l)| {
        matches!(
            l.content,
            LineContent::DiffHunk(_) | LineContent::DiffLine(_)
        )
    });
    if all_hunks {
        // Collect unique (path, hunk_index) pairs
        let hunk_info: Vec<(&str, usize)> = selected
            .iter()
            .filter_map(|(_, l)| match &l.section {
                Some(SectionType::StagedHunk { path, hunk_index }) => {
                    Some((path.as_str(), *hunk_index))
                }
                _ => None,
            })
            .collect();

        if !hunk_info.is_empty() {
            // Check if all hunks are from the same file
            let first_path = hunk_info[0].0;
            let all_same_file = hunk_info.iter().all(|(p, _)| *p == first_path);

            if all_same_file {
                // Unstage hunks in reverse order to avoid index shifts
                let mut indices: Vec<usize> = hunk_info.iter().map(|(_, i)| *i).collect();
                indices.sort_unstable();
                indices.dedup();
                // Unstage from highest index first so earlier indices remain valid
                for &idx in indices.iter().rev() {
                    unstage_hunk(repo_path, first_path, idx)?;
                }
                return Ok(());
            }
        }
    }

    // Fallback: treat as normal mode on cursor position
    handle_normal_mode(model, repo_path)
}

/// Computes the 0-based indices of diff lines within a hunk that are selected.
/// Counts only DiffLine entries (not the hunk header) in the matching section.
fn compute_hunk_line_indices(
    lines: &[crate::model::Line],
    sel_start: usize,
    sel_end: usize,
    target_path: &str,
    target_hunk_index: usize,
) -> Vec<usize> {
    let mut result = Vec::new();
    let mut line_idx_in_hunk: usize = 0;

    for (i, line) in lines.iter().enumerate() {
        let is_matching_section = matches!(
            &line.section,
            Some(SectionType::StagedHunk { path, hunk_index })
                if path == target_path && *hunk_index == target_hunk_index
        );

        if !is_matching_section {
            continue;
        }

        if matches!(line.content, LineContent::DiffLine(_)) {
            if i >= sel_start && i <= sel_end {
                result.push(line_idx_in_hunk);
            }
            line_idx_in_hunk += 1;
        }
    }

    result
}
