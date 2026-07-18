use std::path::Path;
use std::process::Stdio;

use super::git_cmd;
use crate::errors::{MagiError, MagiResult};

/// Applies a patch to the working tree by piping it to `git apply`.
pub fn apply_patch<P: AsRef<Path>>(repo_path: P, patch: &str) -> MagiResult<()> {
    use std::io::Write;
    let mut child = git_cmd(&repo_path, &["apply"])
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
        return Err(MagiError::Generic(format!("git apply failed: {}", stderr)));
    }
    Ok(())
}
