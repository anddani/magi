use std::process::Stdio;
use std::time::Instant;

use crate::{
    git::git_cmd,
    model::{Model, Toast, ToastStyle, popup::PopupContent},
    msg::Message,
    msg::update::commit::TOAST_DURATION,
};

/// Prune tags: delete local-only tags with `git tag -d` and push-delete
/// remote-only tags with `git push <remote> :tag1 :tag2 ...`.
pub fn update(
    model: &mut Model,
    local_tags: Vec<String>,
    remote_tags: Vec<String>,
    remote: String,
) -> Option<Message> {
    model.popup = None;

    // Delete local tags
    if !local_tags.is_empty() {
        let mut args = vec!["tag", "-d"];
        let tag_refs: Vec<&str> = local_tags.iter().map(|s| s.as_str()).collect();
        args.extend(tag_refs);

        let output = git_cmd(&model.workdir, &args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(out) if !out.status.success() => {
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                model.popup = Some(PopupContent::Error {
                    message: format!("Failed to delete local tags: {}", stderr),
                });
                return None;
            }
            Err(err) => {
                model.popup = Some(PopupContent::Error {
                    message: format!("Failed to delete local tags: {}", err),
                });
                return None;
            }
            _ => {}
        }
    }

    // Push-delete remote tags: git push <remote> :tag1 :tag2 ...
    if !remote_tags.is_empty() {
        let refspecs: Vec<String> = remote_tags.iter().map(|t| format!(":{}", t)).collect();
        let mut args = vec!["push", remote.as_str()];
        let refspec_args: Vec<&str> = refspecs.iter().map(|s| s.as_str()).collect();
        args.extend(refspec_args);

        let output = git_cmd(&model.workdir, &args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(out) if !out.status.success() => {
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                model.popup = Some(PopupContent::Error {
                    message: format!("Failed to delete remote tags from '{}': {}", remote, stderr),
                });
                return None;
            }
            Err(err) => {
                model.popup = Some(PopupContent::Error {
                    message: format!("Failed to delete remote tags from '{}': {}", remote, err),
                });
                return None;
            }
            _ => {}
        }
    }

    model.toast = Some(Toast {
        message: format!("Pruned tags against '{}'", remote),
        style: ToastStyle::Success,
        expires_at: Instant::now() + TOAST_DURATION,
    });

    Some(Message::Refresh)
}
