use std::io::Write;
use std::path::Path;
use std::process::Stdio;

use super::git_cmd;
use super::stage::{extract_hunk_from_diff, get_file_diff, parse_hunk_header_starts};
use crate::errors::{MagiError, MagiResult};

/// Discards untracked files by running `git clean -f -- <files>`.
/// If `files` is empty, this is a no-op.
pub fn discard_untracked_files<P: AsRef<Path>>(repo_path: P, files: &[&str]) -> MagiResult<()> {
    if files.is_empty() {
        return Ok(());
    }
    let output = git_cmd(&repo_path, &["clean", "-f", "--"])
        .args(files)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(MagiError::Generic(format!(
            "git clean -f failed: {}",
            stderr
        )));
    }
    Ok(())
}

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

// ============================================================================
// Staged discard operations
// ============================================================================

/// Discards staged files. For new files, this removes them entirely (from index and disk).
/// For modified files, this applies the reverse patch to BOTH index and working tree,
/// preserving any other unstaged changes in those files.
pub fn discard_staged_files<P: AsRef<Path>>(repo_path: P, files: &[&str]) -> MagiResult<()> {
    if files.is_empty() {
        return Ok(());
    }

    for file in files {
        if is_file_new_in_index(&repo_path, file)? {
            // New file: remove from index and delete from disk
            let output = git_cmd(&repo_path, &["rm", "-f", "--", file])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(MagiError::Generic(format!("git rm -f failed: {}", stderr)));
            }
        } else {
            // Modified file: get the staged diff and apply reverse to both index and working tree
            let diff_output = get_staged_file_diff(&repo_path, file)?;
            if diff_output.trim().is_empty() {
                continue; // No staged changes for this file
            }
            // Apply reverse to both index and working tree
            apply_patch_reverse_index_and_worktree(&repo_path, &diff_output)?;
        }
    }
    Ok(())
}

/// Checks if a file is new in the index (not present in HEAD).
fn is_file_new_in_index<P: AsRef<Path>>(repo_path: P, file: &str) -> MagiResult<bool> {
    // Try to show the file in HEAD. If it fails, the file is new.
    let output = git_cmd(&repo_path, &["cat-file", "-e", &format!("HEAD:{}", file)])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    Ok(!output.status.success())
}

/// Discards a single staged hunk by applying the reverse patch to both index and working tree.
/// This removes the staged changes while preserving any other unstaged changes.
pub fn discard_staged_hunk<P: AsRef<Path>>(
    repo_path: P,
    file: &str,
    hunk_index: usize,
) -> MagiResult<()> {
    let patch = extract_staged_hunk_patch(&repo_path, file, hunk_index)?;
    apply_patch_reverse_index_and_worktree(&repo_path, &patch)
}

/// Discards specific staged lines within a hunk.
pub fn discard_staged_lines<P: AsRef<Path>>(
    repo_path: P,
    file: &str,
    hunk_index: usize,
    selected_line_indices: &[usize],
) -> MagiResult<()> {
    let diff_output = get_staged_file_diff(&repo_path, file)?;
    let (header, hunk_lines) = extract_hunk_from_diff(&diff_output, hunk_index)?;

    // Build modified hunk with only selected lines discarded
    let mut modified_lines: Vec<String> = Vec::new();
    let content_lines: Vec<&str> = hunk_lines.iter().skip(1).cloned().collect();

    let mut old_count: usize = 0;
    let mut new_count: usize = 0;

    for (i, line) in content_lines.iter().enumerate() {
        let is_selected = selected_line_indices.contains(&i);
        if let Some(stripped) = line.strip_prefix('+') {
            if is_selected {
                modified_lines.push(line.to_string());
                new_count += 1;
            } else {
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
        } else {
            modified_lines.push(line.to_string());
            old_count += 1;
            new_count += 1;
        }
    }

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

    apply_patch_reverse_index_and_worktree(&repo_path, &patch)
}

/// Gets the staged diff output for a specific file (comparing index to HEAD).
fn get_staged_file_diff<P: AsRef<Path>>(repo_path: P, file: &str) -> MagiResult<String> {
    let output = git_cmd(&repo_path, &["diff", "--cached", "--", file])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Extracts a single staged hunk patch.
fn extract_staged_hunk_patch<P: AsRef<Path>>(
    repo_path: P,
    file: &str,
    hunk_index: usize,
) -> MagiResult<String> {
    let diff_output = get_staged_file_diff(&repo_path, file)?;
    let (header, hunk_lines) = extract_hunk_from_diff(&diff_output, hunk_index)?;

    let mut patch = header;
    patch.push('\n');
    patch.push_str(&hunk_lines.join("\n"));
    patch.push('\n');

    Ok(patch)
}

/// Applies a patch in reverse to BOTH the index and working tree.
/// First tries `--index` (which requires working tree to match index for affected paths).
/// When there are additional unstaged changes, falls back to:
/// 1. Saving the unstaged diff (with zero context to avoid context mismatch)
/// 2. Resetting affected files to HEAD (both index and working tree)
/// 3. Re-applying only the unstaged changes
fn apply_patch_reverse_index_and_worktree<P: AsRef<Path>>(
    repo_path: P,
    patch: &str,
) -> MagiResult<()> {
    // Try --index first (applies to both index and working tree atomically,
    // but requires working tree to match index for affected paths)
    let mut child = git_cmd(&repo_path, &["apply", "--reverse", "--index"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(patch.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if output.status.success() {
        return Ok(());
    }

    // Fallback: working tree has additional unstaged changes.
    // Extract affected file paths from the patch, save their unstaged diffs,
    // reset to HEAD, then re-apply only the unstaged changes.
    let files = extract_files_from_patch(patch);

    // Save unstaged diffs (with zero context to avoid mismatch after reset)
    let mut unstaged_patches: Vec<String> = Vec::new();
    for file in &files {
        let output = git_cmd(&repo_path, &["diff", "-U0", "--", file])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        let diff = String::from_utf8_lossy(&output.stdout).to_string();
        if !diff.trim().is_empty() {
            unstaged_patches.push(diff);
        }
    }

    // Reset affected files to HEAD (both index and working tree)
    let mut checkout_args = vec!["checkout", "HEAD", "--"];
    for file in &files {
        checkout_args.push(file);
    }
    let output = git_cmd(&repo_path, &checkout_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(MagiError::Generic(format!(
            "git checkout HEAD failed: {}",
            stderr
        )));
    }

    // Re-apply only the unstaged changes
    for unstaged_patch in &unstaged_patches {
        let mut child = git_cmd(&repo_path, &["apply", "--unidiff-zero"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(unstaged_patch.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(MagiError::Generic(format!(
                "git apply unstaged changes failed: {}",
                stderr
            )));
        }
    }

    Ok(())
}

/// Extracts file paths from a unified diff patch.
fn extract_files_from_patch(patch: &str) -> Vec<String> {
    let mut files = Vec::new();
    for line in patch.lines() {
        if let Some(path) = line.strip_prefix("+++ b/") {
            files.push(path.to_string());
        }
    }
    files
}
