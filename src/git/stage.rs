use std::path::Path;
use std::process::Stdio;

use super::git_cmd;
use crate::errors::{MagiError, MagiResult};

/// Stages the specified files.
/// If `files` is empty, this is a no-op.
pub fn stage_files<P: AsRef<Path>>(repo_path: P, files: &[&str]) -> MagiResult<()> {
    if files.is_empty() {
        return Ok(());
    }
    let _output = git_cmd(&repo_path, &["add", "--"])
        .args(files)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    Ok(())
}

/// Unstages the specified files.
/// If `files` is empty, this is a no-op.
pub fn unstage_files<P: AsRef<Path>>(repo_path: P, files: &[&str]) -> MagiResult<()> {
    if files.is_empty() {
        return Ok(());
    }
    let _output = git_cmd(&repo_path, &["reset", "HEAD", "--"])
        .args(files)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    Ok(())
}

/// Stages a single hunk from a file by extracting it from `git diff` output
/// and piping it to `git apply --cached`.
pub fn stage_hunk<P: AsRef<Path>>(repo_path: P, file: &str, hunk_index: usize) -> MagiResult<()> {
    let patch = extract_hunk_patch(&repo_path, file, hunk_index)?;
    apply_patch_cached(&repo_path, &patch)
}

/// Stages specific lines within a hunk. Lines not in the selection are converted:
/// - `+` lines not selected are removed from the patch
/// - `-` lines not selected become context lines (prefix changed to ` `)
///   `selected_line_indices` are 0-based indices within the hunk's diff lines (not counting the hunk header).
pub fn stage_lines<P: AsRef<Path>>(
    repo_path: P,
    file: &str,
    hunk_index: usize,
    selected_line_indices: &[usize],
) -> MagiResult<()> {
    let diff_output = get_file_diff(&repo_path, file)?;
    let (header, hunk_lines) = extract_hunk_from_diff(&diff_output, hunk_index)?;

    // Build modified hunk with only selected lines staged
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
        if line.starts_with('+') {
            if is_selected {
                modified_lines.push(line.to_string());
                new_count += 1;
            }
            // Unselected additions are simply omitted
        } else if let Some(stripped) = line.strip_prefix('-') {
            if is_selected {
                modified_lines.push(line.to_string());
                old_count += 1;
            } else {
                // Convert unselected deletion to context line
                let context = format!(" {}", stripped);
                modified_lines.push(context);
                old_count += 1;
                new_count += 1;
            }
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

    apply_patch_cached(&repo_path, &patch)
}

/// Gets the diff output for a specific file.
fn get_file_diff<P: AsRef<Path>>(repo_path: P, file: &str) -> MagiResult<String> {
    let output = git_cmd(&repo_path, &["diff", "--", file])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Extracts the diff file header and lines of a specific hunk from diff output.
/// Returns (file_header, hunk_lines) where hunk_lines includes the @@ header.
fn extract_hunk_from_diff(diff_output: &str, hunk_index: usize) -> MagiResult<(String, Vec<&str>)> {
    let lines: Vec<&str> = diff_output.lines().collect();

    // Find the file header (everything before the first @@ line)
    let first_hunk_pos = lines
        .iter()
        .position(|l| l.starts_with("@@"))
        .ok_or_else(|| MagiError::Generic("No hunks found in diff output".to_string()))?;

    let header = lines[..first_hunk_pos].join("\n");

    // Find all hunk start positions
    let hunk_starts: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.starts_with("@@"))
        .map(|(i, _)| i)
        .collect();

    if hunk_index >= hunk_starts.len() {
        return Err(MagiError::Generic(format!(
            "Hunk index {} out of range (file has {} hunks)",
            hunk_index,
            hunk_starts.len()
        )));
    }

    let start = hunk_starts[hunk_index];
    let end = hunk_starts
        .get(hunk_index + 1)
        .copied()
        .unwrap_or(lines.len());

    let hunk_lines = lines[start..end].to_vec();

    Ok((header, hunk_lines))
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

/// Applies a patch to the staging area.
fn apply_patch_cached<P: AsRef<Path>>(repo_path: P, patch: &str) -> MagiResult<()> {
    use std::io::Write;
    let mut child = git_cmd(&repo_path, &["apply", "--cached"])
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
            "git apply --cached failed: {}",
            stderr
        )));
    }
    Ok(())
}

/// Unstages a single hunk from a file by extracting it from `git diff --cached` output
/// and piping it to `git apply --cached --reverse`.
pub fn unstage_hunk<P: AsRef<Path>>(repo_path: P, file: &str, hunk_index: usize) -> MagiResult<()> {
    let patch = extract_staged_hunk_patch(&repo_path, file, hunk_index)?;
    apply_patch_cached_reverse(&repo_path, &patch)
}

/// Unstages specific lines within a staged hunk. Lines not in the selection are converted:
/// - `+` lines not selected become context lines (prefix changed to ` `)
/// - `-` lines not selected are removed from the patch
///
/// `selected_line_indices` are 0-based indices within the hunk's diff lines (not counting the hunk header).
pub fn unstage_lines<P: AsRef<Path>>(
    repo_path: P,
    file: &str,
    hunk_index: usize,
    selected_line_indices: &[usize],
) -> MagiResult<()> {
    let diff_output = get_staged_file_diff(&repo_path, file)?;
    let (header, hunk_lines) = extract_hunk_from_diff(&diff_output, hunk_index)?;

    // Build modified hunk with only selected lines unstaged
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

    apply_patch_cached_reverse(&repo_path, &patch)
}

/// Gets the staged diff output for a specific file.
fn get_staged_file_diff<P: AsRef<Path>>(repo_path: P, file: &str) -> MagiResult<String> {
    let output = git_cmd(&repo_path, &["diff", "--cached", "--", file])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Extracts a single staged hunk patch (file header + hunk) ready to be applied in reverse.
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

/// Applies a patch in reverse to the staging area (unstages changes).
fn apply_patch_cached_reverse<P: AsRef<Path>>(repo_path: P, patch: &str) -> MagiResult<()> {
    use std::io::Write;
    let mut child = git_cmd(&repo_path, &["apply", "--cached", "--reverse"])
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
            "git apply --cached --reverse failed: {}",
            stderr
        )));
    }
    Ok(())
}

/// Parses a hunk header like "@@ -7,6 +7,8 @@" to extract (old_start, new_start).
fn parse_hunk_header_starts(header: &str) -> MagiResult<(usize, usize)> {
    // Format: @@ -old_start[,old_count] +new_start[,new_count] @@
    let parts: Vec<&str> = header.split_whitespace().collect();
    if parts.len() < 4 {
        return Err(MagiError::Generic(format!(
            "Invalid hunk header: {}",
            header
        )));
    }
    let old_part = parts[1]; // e.g. "-7,6"
    let new_part = parts[2]; // e.g. "+7,8"

    let old_start: usize = old_part
        .trim_start_matches('-')
        .split(',')
        .next()
        .unwrap_or("1")
        .parse()
        .map_err(|_| MagiError::Generic(format!("Cannot parse old start from: {}", old_part)))?;

    let new_start: usize = new_part
        .trim_start_matches('+')
        .split(',')
        .next()
        .unwrap_or("1")
        .parse()
        .map_err(|_| MagiError::Generic(format!("Cannot parse new start from: {}", new_part)))?;

    Ok((old_start, new_start))
}
