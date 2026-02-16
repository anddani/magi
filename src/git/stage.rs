use std::path::Path;
use std::process::Stdio;

use super::git_cmd;
use crate::errors::MagiResult;

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
