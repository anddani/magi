use std::path::Path;

use super::discard::{apply_patch_reverse, build_partial_hunk_patch, get_staged_file_diff};
use super::stage::extract_hunk_from_diff;
use crate::errors::MagiResult;

/// Reverses a patch in the working tree by piping it to `git apply --reverse`.
/// The index is left untouched, mirroring magit's `magit-reverse`.
pub fn reverse_patch<P: AsRef<Path>>(repo_path: P, patch: &str) -> MagiResult<()> {
    apply_patch_reverse(repo_path, patch)
}

/// Reverses the staged changes of the given files in the working tree only.
/// The changes stay staged in the index; only the working tree is undone.
pub fn reverse_staged_files<P: AsRef<Path>>(repo_path: P, files: &[&str]) -> MagiResult<()> {
    for file in files {
        let diff_output = get_staged_file_diff(&repo_path, file)?;
        if diff_output.trim().is_empty() {
            continue; // No staged changes for this file
        }
        apply_patch_reverse(&repo_path, &diff_output)?;
    }
    Ok(())
}

/// Reverses a single staged hunk in the working tree only.
pub fn reverse_staged_hunk<P: AsRef<Path>>(
    repo_path: P,
    file: &str,
    hunk_index: usize,
) -> MagiResult<()> {
    reverse_staged_hunks(repo_path, file, &[hunk_index])
}

/// Reverses multiple staged hunks of the same file in the working tree only.
/// All hunks are combined into a single patch so the reversal is atomic and
/// line offsets stay correct.
pub fn reverse_staged_hunks<P: AsRef<Path>>(
    repo_path: P,
    file: &str,
    hunk_indices: &[usize],
) -> MagiResult<()> {
    let diff_output = get_staged_file_diff(&repo_path, file)?;

    let mut indices: Vec<usize> = hunk_indices.to_vec();
    indices.sort_unstable();
    indices.dedup();

    let mut patch = String::new();
    for (i, &hunk_index) in indices.iter().enumerate() {
        let (header, hunk_lines) = extract_hunk_from_diff(&diff_output, hunk_index)?;
        if i == 0 {
            patch.push_str(&header);
            patch.push('\n');
        }
        patch.push_str(&hunk_lines.join("\n"));
        patch.push('\n');
    }

    apply_patch_reverse(&repo_path, &patch)
}

/// Reverses specific lines within a staged hunk in the working tree only.
pub fn reverse_staged_lines<P: AsRef<Path>>(
    repo_path: P,
    file: &str,
    hunk_index: usize,
    selected_line_indices: &[usize],
) -> MagiResult<()> {
    let diff_output = get_staged_file_diff(&repo_path, file)?;
    let patch = build_partial_hunk_patch(&diff_output, hunk_index, selected_line_indices)?;
    apply_patch_reverse(&repo_path, &patch)
}
