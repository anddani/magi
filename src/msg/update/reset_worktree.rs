use std::process::Stdio;

use crate::{
    git::git_cmd,
    model::{Model, popup::PopupContent},
    msg::Message,
};

/// Reset the working tree to match `target` without touching HEAD or the index.
/// Equivalent to `git restore --worktree --source=<target> .`
pub fn update(model: &mut Model, target: String) -> Option<Message> {
    let output = git_cmd(
        &model.workdir,
        &[
            "restore",
            "--worktree",
            &format!("--source={}", target),
            ".",
        ],
    )
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .output();

    model.popup = None;

    match output {
        Ok(out) if out.status.success() => Some(Message::Refresh),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to reset worktree to {}: {}", target, stderr),
            });
            None
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to reset worktree to {}: {}", target, err),
            });
            None
        }
    }
}
