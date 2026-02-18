use std::io::Write;
use std::path::Path;
use std::process::Stdio;

use super::git_cmd;
use super::stage::{extract_hunk_from_diff, get_file_diff, parse_hunk_header_starts};
use crate::errors::{MagiError, MagiResult};

/// Discards changes in the specified files by checking them out from HEAD.
/// If `files` is empty, this is a no-op.
pub fn discard_files<P: AsRef<Path>>(repo_path: P, files: &[&str]) -> MagiResult<()> {
    if files.is_empty() {
        return Ok(());
    }
    let output = git_cmd(&repo_path, &["checkout", "--"])
        .args(files)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(MagiError::Generic(format!(
            "git checkout failed: {}",
            stderr
        )));
    }
    Ok(())
}

/// Discards a single hunk from a file by extracting it from `git diff` output
/// and piping it to `git apply --reverse`.
pub fn discard_hunk<P: AsRef<Path>>(repo_path: P, file: &str, hunk_index: usize) -> MagiResult<()> {
    let patch = extract_hunk_patch(&repo_path, file, hunk_index)?;
    apply_patch_reverse(&repo_path, &patch)
}

/// Discards specific lines within a hunk by applying a modified patch in reverse.
/// Lines not in the selection are converted:
/// - `+` lines not selected become context lines (prefix changed to ` `)
/// - `-` lines not selected are removed from the patch
///
/// `selected_line_indices` are 0-based indices within the hunk's diff lines (not counting the hunk header).
pub fn discard_lines<P: AsRef<Path>>(
    repo_path: P,
    file: &str,
    hunk_index: usize,
    selected_line_indices: &[usize],
) -> MagiResult<()> {
    let diff_output = get_file_diff(&repo_path, file)?;
    let (header, hunk_lines) = extract_hunk_from_diff(&diff_output, hunk_index)?;

    // Build modified hunk with only selected lines discarded
    let mut modified_lines: Vec<String> = Vec::new();
    // Skip the hunk header line (first line), process diff content lines
    let content_lines: Vec<&str> = hunk_lines
        .iter()
        .skip(1) // skip @@ header
        .cloned()
        .collect();

    let mut old_count: usize = 0;
    let mut new_count: usize = 0;

    for (i, line) in content_lines.iter().enumerate() {
        let is_selected = selected_line_indices.contains(&i);
        if let Some(stripped) = line.strip_prefix('+') {
            if is_selected {
                modified_lines.push(line.to_string());
                new_count += 1;
            } else {
                // Convert unselected addition to context line
                let context = format!(" {}", stripped);
                modified_lines.push(context);
                old_count += 1;
                new_count += 1;
            }
        } else if line.starts_with('-') {
            if is_selected {
                modified_lines.push(line.to_string());
                old_count += 1;
            }
            // Unselected deletions are simply omitted
        } else {
            // Context line
            modified_lines.push(line.to_string());
            old_count += 1;
            new_count += 1;
        }
    }

    // Parse original hunk header to get the start lines
    let hunk_header = &hunk_lines[0];
    let (old_start, new_start) = parse_hunk_header_starts(hunk_header)?;

    let new_header = format!(
        "@@ -{},{} +{},{} @@",
        old_start, old_count, new_start, new_count
    );

    let mut patch = header;
    patch.push('\n');
    patch.push_str(&new_header);
    patch.push('\n');
    patch.push_str(&modified_lines.join("\n"));
    patch.push('\n');

    apply_patch_reverse(&repo_path, &patch)
}

/// Extracts a single hunk patch (file header + hunk) ready to be applied.
fn extract_hunk_patch<P: AsRef<Path>>(
    repo_path: P,
    file: &str,
    hunk_index: usize,
) -> MagiResult<String> {
    let diff_output = get_file_diff(&repo_path, file)?;
    let (header, hunk_lines) = extract_hunk_from_diff(&diff_output, hunk_index)?;

    let mut patch = header;
    patch.push('\n');
    patch.push_str(&hunk_lines.join("\n"));
    patch.push('\n');

    Ok(patch)
}

/// Applies a patch in reverse to the working tree (discards changes).
fn apply_patch_reverse<P: AsRef<Path>>(repo_path: P, patch: &str) -> MagiResult<()> {
    let mut child = git_cmd(&repo_path, &["apply", "--reverse"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(patch.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(MagiError::Generic(format!(
            "git apply --reverse failed: {}",
            stderr
        )));
    }
    Ok(())
}
