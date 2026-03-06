use std::process::Stdio;

use crate::{
    git::git_cmd,
    model::{Model, popup::PopupContent},
    msg::{Message, ResetMode},
};

pub fn update(
    model: &mut Model,
    branch: String,
    target: String,
    mode: ResetMode,
) -> Option<Message> {
    let current_branch = model.git_info.current_branch();

    let output = if current_branch.as_deref() == Some(branch.as_str()) {
        // Reset the current branch using the requested mode
        git_cmd(&model.workdir, &["reset", mode.flag(), &target])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    } else {
        // Move a non-checked-out branch to the target without touching the working tree
        git_cmd(&model.workdir, &["branch", "-f", &branch, &target])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    };

    model.popup = None;

    match output {
        Ok(out) if out.status.success() => Some(Message::Refresh),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to reset {} to {}: {}", branch, target, stderr),
            });
            None
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to reset {} to {}: {}", branch, target, err),
            });
            None
        }
    }
}
