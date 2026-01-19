use std::time::{Duration, Instant};

use crate::{
    git::push::{push, PushResult},
    model::{
        popup::{PopupContent, PopupContentCommand},
        Model, Toast, ToastStyle,
    },
    msg::Message,
};

/// Duration for toast notifications
const TOAST_DURATION: Duration = Duration::from_secs(5);

/// Parse input into (remote, branch) tuple.
/// If input contains "/", split on first "/" to get remote and branch.
/// Otherwise, use the default remote and the input as the branch name.
fn parse_remote_branch(input: &str, default_remote: &str, local_branch: &str) -> (String, String) {
    let input = input.trim();
    if input.is_empty() {
        // Use defaults
        (default_remote.to_string(), local_branch.to_string())
    } else if let Some((remote, branch)) = input.split_once('/') {
        // User specified remote/branch
        (remote.to_string(), branch.to_string())
    } else {
        // User specified only branch, use default remote
        (default_remote.to_string(), input.to_string())
    }
}

pub fn update(model: &mut Model) -> Option<Message> {
    // Get the remote and branch to push to
    let (remote, branch) =
        if let Some(PopupContent::Command(PopupContentCommand::Push(ref state))) = model.popup {
            parse_remote_branch(
                &state.input_text,
                &state.default_remote,
                &state.local_branch,
            )
        } else {
            return None;
        };

    // Dismiss the popup
    model.popup = None;

    if let Some(repo_path) = model.git_info.repository.workdir() {
        let refspec = format!("HEAD:{}", branch);
        match push(repo_path, &["--set-upstream", &remote, &refspec]) {
            Ok(PushResult::Success) => {
                let message = format!("Pushed to {}/{}", remote, branch);
                model.toast = Some(Toast {
                    message,
                    style: ToastStyle::Success,
                    expires_at: Instant::now() + TOAST_DURATION,
                });
            }
            Ok(PushResult::Error(message)) => {
                model.popup = Some(PopupContent::Error { message });
            }
            Err(e) => {
                model.popup = Some(PopupContent::Error {
                    message: e.to_string(),
                });
            }
        }
    } else {
        model.popup = Some(PopupContent::Error {
            message: "Repository working directory not found".into(),
        });
    }

    Some(Message::Refresh)
}
