use crate::{
    git::stage::{stage_files, stage_hunk, stage_lines},
    model::{LineContent, Model, SectionType, popup::PopupContent},
    msg::Message,
};

pub fn update(model: &mut Model) -> Option<Message> {
    let repo_path = model.git_info.repository.workdir()?.to_path_buf();

    let result = if model.ui_model.is_visual_mode() {
        handle_visual_mode(model, &repo_path)
    } else {
        handle_normal_mode(model, &repo_path)
    };

    // Exit visual mode after staging
    model.ui_model.visual_mode_anchor = None;

    if let Err(e) = result {
        model.popup = Some(PopupContent::Error {
            message: format!("Error staging: {}", e),
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
        // Section header for untracked files → stage all untracked
        (LineContent::SectionHeader { .. }, Some(SectionType::UntrackedFiles)) => {
            let files: Vec<&str> = lines
                .iter()
                .filter_map(|l| match &l.content {
                    LineContent::UntrackedFile(path) => Some(path.as_str()),
                    _ => None,
                })
                .collect();
            stage_files(repo_path, &files)
        }
        // Section header for unstaged changes → stage all unstaged
        (LineContent::SectionHeader { .. }, Some(SectionType::UnstagedChanges)) => {
            let files: Vec<&str> = lines
                .iter()
                .filter_map(|l| match &l.content {
                    LineContent::UnstagedFile(fc) => Some(fc.path.as_str()),
                    _ => None,
                })
                .collect();
            stage_files(repo_path, &files)
        }
        // Untracked file → stage that file
        (LineContent::UntrackedFile(path), _) => stage_files(repo_path, &[path.as_str()]),
        // Unstaged file → stage that file
        (LineContent::UnstagedFile(fc), _) => stage_files(repo_path, &[fc.path.as_str()]),
        // Diff hunk in unstaged section → stage that hunk
        (
            LineContent::DiffHunk(_),
            Some(SectionType::UnstagedHunk {
                path, hunk_index, ..
            }),
        ) => stage_hunk(repo_path, path, *hunk_index),
        // Diff line in unstaged section → stage the whole hunk it belongs to
        (
            LineContent::DiffLine(_),
            Some(SectionType::UnstagedHunk {
                path, hunk_index, ..
            }),
        ) => stage_hunk(repo_path, path, *hunk_index),
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

    // Try to determine what kind of selection we have
    // Check if all are untracked files
    let all_untracked = selected.iter().all(|(_, l)| {
        matches!(
            l.content,
            LineContent::UntrackedFile(_) | LineContent::SectionHeader { .. }
        )
    });
    if all_untracked {
        let files: Vec<&str> = selected
            .iter()
            .filter_map(|(_, l)| match &l.content {
                LineContent::UntrackedFile(path) => Some(path.as_str()),
                _ => None,
            })
            .collect();
        if !files.is_empty() {
            return stage_files(repo_path, &files);
        }
    }

    // Check if all are unstaged files
    let all_unstaged_files = selected.iter().all(|(_, l)| {
        matches!(
            l.content,
            LineContent::UnstagedFile(_) | LineContent::SectionHeader { .. }
        )
    });
    if all_unstaged_files {
        let files: Vec<&str> = selected
            .iter()
            .filter_map(|(_, l)| match &l.content {
                LineContent::UnstagedFile(fc) => Some(fc.path.as_str()),
                _ => None,
            })
            .collect();
        if !files.is_empty() {
            return stage_files(repo_path, &files);
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

        if let Some(SectionType::UnstagedHunk { path, hunk_index }) = sections.first() {
            let all_same_hunk = sections.iter().all(|s| {
                matches!(s, SectionType::UnstagedHunk { path: p, hunk_index: h } if p == path && h == hunk_index)
            });

            if all_same_hunk {
                // Calculate which lines within the hunk are selected
                let line_indices = compute_hunk_line_indices(lines, start, end, path, *hunk_index);
                if !line_indices.is_empty() {
                    return stage_lines(repo_path, path, *hunk_index, &line_indices);
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
                Some(SectionType::UnstagedHunk { path, hunk_index }) => {
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
                // Stage hunks in reverse order to avoid index shifts
                let mut indices: Vec<usize> = hunk_info.iter().map(|(_, i)| *i).collect();
                indices.sort_unstable();
                indices.dedup();
                // Stage from highest index first so earlier indices remain valid
                for &idx in indices.iter().rev() {
                    stage_hunk(repo_path, first_path, idx)?;
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
            Some(SectionType::UnstagedHunk { path, hunk_index })
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
