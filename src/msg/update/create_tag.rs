use std::process::Stdio;

use crate::{
    git::git_cmd,
    model::{Model, popup::PopupContent},
    msg::Message,
};

/// Create a new git tag pointing at `target`.
/// Equivalent to `git tag <name> <target>`.
pub fn update(model: &mut Model, name: String, target: String) -> Option<Message> {
    let output = git_cmd(&model.workdir, &["tag", &name, &target])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    model.popup = None;

    match output {
        Ok(out) if out.status.success() => Some(Message::Refresh),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to create tag '{}': {}", name, stderr),
            });
            None
        }
        Err(err) => {
            model.popup = Some(PopupContent::Error {
                message: format!("Failed to create tag '{}': {}", name, err),
            });
            None
        }
    }
}
