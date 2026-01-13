use std::path::Path;
use std::process::{Command, Stdio};

use crate::errors::MagiResult;

/// Stages the specified files.
/// If `files` is empty, this is a no-op.
pub fn stage_files<P: AsRef<Path>>(repo_path: P, files: &[&str]) -> MagiResult<()> {
    if files.is_empty() {
        return Ok(());
    }
    let _output = Command::new("git")
        .arg("-C")
        .arg(repo_path.as_ref())
        .arg("add")
        .arg("--")
        .args(files)
        .stdout(Stdio::piped())
        .output()?;
    Ok(())
}

/// Unstages the specified files.
/// If `files` is empty, this is a no-op.
pub fn unstage_files<P: AsRef<Path>>(repo_path: P, files: &[&str]) -> MagiResult<()> {
    if files.is_empty() {
        return Ok(());
    }
    let _output = Command::new("git")
        .arg("-C")
        .arg(repo_path.as_ref())
        .arg("reset")
        .arg("HEAD")
        .arg("--")
        .args(files)
        .stdout(Stdio::piped())
        .output()?;
    Ok(())
}
